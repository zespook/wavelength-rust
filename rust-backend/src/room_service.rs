use super::{clean, normalize_code, now, random_token, token_hash, Party, PartyAuth};
use mongodb::{
    bson::{doc, Bson},
    Collection,
};
use serde::Serialize;
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    sync::Arc,
};
use tokio::sync::{broadcast, Mutex};

#[derive(Clone)]
pub(super) struct RoomService {
    parties: Collection<Party>,
    party_auth: Collection<PartyAuth>,
    audio_root: PathBuf,
    channels: Arc<Mutex<HashMap<String, broadcast::Sender<String>>>>,
    presence: LivePresence,
    socket_tickets: SocketTicketStore,
}

pub(super) struct RoomSocketTicket {
    pub(super) code: String,
    pub(super) username: String,
    pub(super) session_expires_at: i64,
    pub(super) is_host: bool,
    expires_at: i64,
}

#[derive(Clone, Default)]
struct SocketTicketStore {
    tickets: Arc<Mutex<HashMap<String, RoomSocketTicket>>>,
}

impl SocketTicketStore {
    async fn create(
        &self,
        code: &str,
        username: &str,
        session_expires_at: i64,
        is_host: bool,
    ) -> String {
        let current_time = now();
        let ticket = random_token(48);
        let mut tickets = self.tickets.lock().await;
        tickets.retain(|_, ticket| ticket.expires_at > current_time);
        tickets.insert(
            token_hash(&ticket),
            RoomSocketTicket {
                code: normalize_code(code),
                username: clean(username),
                session_expires_at,
                is_host,
                expires_at: current_time + 60_000,
            },
        );
        ticket
    }

    async fn consume(&self, ticket: &str) -> Option<RoomSocketTicket> {
        if ticket.is_empty() || ticket.len() > 128 {
            return None;
        }
        let mut tickets = self.tickets.lock().await;
        let current_time = now();
        tickets.retain(|_, ticket| ticket.expires_at > current_time);
        tickets.remove(&token_hash(ticket))
    }
}

#[derive(Clone, Default)]
struct LivePresence {
    sockets: Arc<Mutex<HashMap<String, HashMap<String, usize>>>>,
}

impl LivePresence {
    async fn connect(&self, code: &str, username: &str) {
        let mut sockets = self.sockets.lock().await;
        *sockets
            .entry(code.to_string())
            .or_default()
            .entry(username.to_string())
            .or_default() += 1;
    }

    async fn disconnect(&self, code: &str, username: &str) -> (bool, bool) {
        let mut sockets = self.sockets.lock().await;
        let Some(users) = sockets.get_mut(code) else {
            return (false, false);
        };
        let last_user_socket = if let Some(count) = users.get_mut(username) {
            *count = count.saturating_sub(1);
            if *count == 0 {
                users.remove(username);
                true
            } else {
                false
            }
        } else {
            false
        };
        let empty = users.is_empty();
        if empty {
            sockets.remove(code);
        }
        (empty, last_user_socket)
    }

    async fn has_live_sockets(&self, code: &str) -> bool {
        self.sockets
            .lock()
            .await
            .get(code)
            .is_some_and(|users| !users.is_empty())
    }
}

impl RoomService {
    pub(super) fn new(
        parties: Collection<Party>,
        party_auth: Collection<PartyAuth>,
        audio_root: PathBuf,
    ) -> Self {
        Self {
            parties,
            party_auth,
            audio_root,
            channels: Arc::new(Mutex::new(HashMap::new())),
            presence: LivePresence::default(),
            socket_tickets: SocketTicketStore::default(),
        }
    }

    pub(super) fn parties(&self) -> &Collection<Party> {
        &self.parties
    }

    pub(super) fn party_auth(&self) -> &Collection<PartyAuth> {
        &self.party_auth
    }

    pub(super) fn audio_root(&self) -> &Path {
        &self.audio_root
    }

    pub(super) async fn create_socket_ticket(
        &self,
        code: &str,
        username: &str,
        session_expires_at: i64,
        is_host: bool,
    ) -> String {
        self.socket_tickets.create(code, username, session_expires_at, is_host).await
    }

    pub(super) async fn consume_socket_ticket(&self, ticket: &str) -> Option<RoomSocketTicket> {
        self.socket_tickets.consume(ticket).await
    }

    pub(super) async fn subscribe(&self, code: &str) -> broadcast::Receiver<String> {
        let mut channels = self.channels.lock().await;
        channels
            .entry(normalize_code(code))
            .or_insert_with(|| broadcast::channel(128).0)
            .subscribe()
    }

    pub(super) async fn publish(&self, code: &str, event: &str, data: impl Serialize) {
        self.publish_from(code, event, data, None).await;
    }

    pub(super) async fn publish_from(
        &self,
        code: &str,
        event: &str,
        data: impl Serialize,
        sender: Option<&str>,
    ) {
        let data = match serde_json::to_value(data) {
            Ok(data) => data,
            Err(error) => {
                tracing::error!(room_code = %normalize_code(code), event, %error, "Room event could not be serialized");
                return;
            }
        };
        let message = match serde_json::to_string(&serde_json::json!({
            "event": event,
            "data": data,
            "sender": sender,
        })) {
            Ok(message) => message,
            Err(error) => {
                tracing::error!(room_code = %normalize_code(code), event, %error, "Room event envelope could not be serialized");
                return;
            }
        };
        let channels = self.channels.lock().await;
        if let Some(channel) = channels.get(&normalize_code(code)) {
            let _ = channel.send(message);
        }
    }

    pub(super) async fn connected(
        &self,
        code: &str,
        username: &str,
    ) -> Result<bool, mongodb::error::Error> {
        let code = normalize_code(code);
        let username = clean(username);
        self.presence.connect(&code, &username).await;
        let result = self
            .parties
            .update_one(
                doc! { "code": &code, "members": &username },
                doc! { "$set": { "emptySince": Bson::Null }, "$addToSet": { "members": &username } },
            )
            .await;
        match result {
            Ok(result) if result.matched_count == 1 => Ok(true),
            Ok(_) => {
                self.presence.disconnect(&code, &username).await;
                Ok(false)
            }
            Err(error) => {
                self.presence.disconnect(&code, &username).await;
                Err(error)
            }
        }
    }

    pub(super) async fn disconnected(&self, code: &str, username: &str) {
        let code = normalize_code(code);
        let username = clean(username);
        let (empty, _) = self.presence.disconnect(&code, &username).await;
        if !empty {
            return;
        }
        if let Err(error) = self
            .parties
            .update_one(
                doc! { "code": &code },
                doc! { "$set": { "emptySince": now() } },
            )
            .await
        {
            tracing::error!(room_code = %code, %error, "Room empty timer could not be updated");
        }
    }

    pub(super) async fn has_live_sockets(&self, code: &str) -> bool {
        self.presence.has_live_sockets(&normalize_code(code)).await
    }
}

#[cfg(test)]
mod tests {
    use super::{LivePresence, SocketTicketStore};

    #[tokio::test]
    async fn room_presence_tracks_each_live_socket_and_username() {
        let presence = LivePresence::default();
        assert!(!presence.has_live_sockets("AB12").await);

        presence.connect("AB12", "Host").await;
        presence.connect("AB12", "Guest").await;
        presence.connect("AB12", "Guest").await;
        assert!(presence.has_live_sockets("AB12").await);
        assert_eq!(presence.disconnect("AB12", "Guest").await, (false, false));
        assert!(presence.has_live_sockets("AB12").await);
        assert_eq!(presence.disconnect("AB12", "Guest").await, (false, true));
        assert_eq!(presence.disconnect("AB12", "Host").await, (true, true));
        assert!(!presence.has_live_sockets("AB12").await);
    }

    #[tokio::test]
    async fn socket_tickets_are_short_lived_and_single_use() {
        let tickets = SocketTicketStore::default();
        let ticket = tickets.create("ab12", " Guest ", 5_000, false).await;
        let payload = tickets.consume(&ticket).await.unwrap();
        assert_eq!(payload.code, "AB12");
        assert_eq!(payload.username, "Guest");
        assert_eq!(payload.session_expires_at, 5_000);
        assert!(!payload.is_host);
        assert!(tickets.consume(&ticket).await.is_none());
    }
}
