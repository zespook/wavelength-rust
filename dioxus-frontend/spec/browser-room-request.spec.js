import assert from "node:assert/strict";
import test from "node:test";
import {
  connectRoomSocket,
  disconnectRoomSocket,
  drainRoomEvents,
  emitRoomHostState,
  emitRoomCommand,
  roomRequest
} from "../src/browser.js";

test("roomRequest sends serialized JSON once and omits bodies on GET", async () => {
  const originalFetch = globalThis.fetch;
  const requests = [];
  globalThis.fetch = async (url, options) => {
    requests.push({ url, options });
    return new Response('{"ok":true}', {
      headers: { "Content-Type": "application/json" }
    });
  };

  try {
    await roomRequest(
      "https://rooms.example/",
      "/session",
      "POST",
      '{"passcode":"test"}',
      "",
      ""
    );
    await roomRequest("https://rooms.example", "/rooms/ABCD", "GET", "{}", "", "");

    assert.equal(requests[0].url, "https://rooms.example/api/v2/session");
    assert.equal(requests[1].url, "https://rooms.example/api/v2/rooms/ABCD");
    assert.equal(requests[0].options.body, '{"passcode":"test"}');
    assert.equal("body" in requests[1].options, false);
  } finally {
    globalThis.fetch = originalFetch;
  }
});

test("room sockets use a one-time ticket and direct JSON events", async () => {
  const OriginalWebSocket = globalThis.WebSocket;
  const originalWindow = globalThis.window;
  const originalFetch = globalThis.fetch;
  const requests = [];
  const sockets = [];
  class MockWebSocket {
    static OPEN = 1;
    readyState = 0;
    sent = [];
    constructor(url) {
      this.url = String(url);
      sockets.push(this);
    }
    send(message) {
      this.sent.push(message);
    }
    close() {
      this.readyState = 3;
    }
  }
  globalThis.WebSocket = MockWebSocket;
  globalThis.window = {
    setTimeout: () => 1,
    clearTimeout: () => {}
  };
  globalThis.fetch = async (url, options) => {
    requests.push({ url, options });
    return new Response('{"ticket":"one-time-ticket"}', {
      headers: { "Content-Type": "application/json" }
    });
  };

  try {
    connectRoomSocket("https://rooms.example", "AB12", "Guest", "access", "", "guest", {
      canManageQueue: true
    });
    connectRoomSocket("https://rooms.example", "AB12", "Guest", "access", "", "guest", {
      canManageQueue: true
    });
    await new Promise(resolve => setImmediate(resolve));
    assert.equal(requests.length, 1);
    const socket = sockets[0];
    const endpoint = new URL(socket.url);
    assert.equal(endpoint.pathname, "/api/v2/rooms/AB12/socket");
    assert.equal(endpoint.protocol, "wss:");
    assert.equal(endpoint.searchParams.get("ticket"), "one-time-ticket");
    assert.equal(endpoint.searchParams.has("accessToken"), false);
    assert.equal(requests[0].url, "https://rooms.example/api/v2/rooms/AB12/socket-ticket");
    assert.equal(requests[0].options.headers["x-room-access-token"], "access");

    socket.readyState = MockWebSocket.OPEN;
    socket.onopen();
    socket.onmessage({
      data: '{"event":"connected","data":{"socketId":"socket-1"}}'
    });
    assert.equal(emitRoomCommand("clear-queue", {}), "sent");
    assert.equal(emitRoomHostState({ queue: [] }), "not-host");
    assert.deepEqual(JSON.parse(socket.sent[0]), {
      event: "room-command",
      data: { action: "clear-queue" }
    });
    socket.onmessage({
      data: '{"event":"room-state","data":{"code":"AB12","permissions":{"canAdd":true}},"sender":"socket-2"}'
    });
    assert.ok(drainRoomEvents().some(event => event.event === "room-state"));
  } finally {
    disconnectRoomSocket();
    globalThis.WebSocket = OriginalWebSocket;
    globalThis.window = originalWindow;
    globalThis.fetch = originalFetch;
  }
});

test("host room state is sent over the active socket", async () => {
  const OriginalWebSocket = globalThis.WebSocket;
  const originalWindow = globalThis.window;
  const originalFetch = globalThis.fetch;
  const sockets = [];
  class MockWebSocket {
    static OPEN = 1;
    readyState = MockWebSocket.OPEN;
    sent = [];
    constructor(url) {
      this.url = String(url);
      sockets.push(this);
    }
    send(message) {
      this.sent.push(message);
    }
    close() {
      this.readyState = 3;
    }
  }
  globalThis.WebSocket = MockWebSocket;
  globalThis.window = {
    setTimeout: () => 1,
    clearTimeout: () => {}
  };
  globalThis.fetch = async () => new Response('{"ticket":"host-ticket"}', {
    headers: { "Content-Type": "application/json" }
  });

  try {
    connectRoomSocket("https://rooms.example", "AB12", "Host", "access", "host-token", "host", {});
    await new Promise(resolve => setImmediate(resolve));
    const socket = sockets[0];
    socket.onopen();
    assert.equal(emitRoomHostState({ queue: [] }), "sent");
    assert.deepEqual(JSON.parse(socket.sent[0]), {
      event: "host-room-state",
      data: { queue: [] }
    });
  } finally {
    disconnectRoomSocket();
    globalThis.WebSocket = OriginalWebSocket;
    globalThis.window = originalWindow;
    globalThis.fetch = originalFetch;
  }
});

test("socket-ticket retries honor the server Retry-After header", async () => {
  const originalWindow = globalThis.window;
  const originalFetch = globalThis.fetch;
  const delays = [];
  globalThis.window = {
    setTimeout: (_callback, delay) => {
      delays.push(delay);
      return 1;
    },
    clearTimeout: () => {}
  };
  globalThis.fetch = async () => new Response('{"error":"Too many attempts"}', {
    status: 429,
    headers: { "Content-Type": "application/json", "Retry-After": "12" }
  });

  try {
    connectRoomSocket("https://rooms.example", "AB12", "Guest", "access", "", "guest", {});
    await new Promise(resolve => setImmediate(resolve));
    assert.deepEqual(delays, [12000]);
  } finally {
    disconnectRoomSocket();
    globalThis.window = originalWindow;
    globalThis.fetch = originalFetch;
  }
});

test("missing rooms end a saved session instead of retrying forever", async () => {
  const originalWindow = globalThis.window;
  const originalFetch = globalThis.fetch;
  let retryScheduled = false;
  globalThis.window = {
    setTimeout: () => {
      retryScheduled = true;
      return 1;
    },
    clearTimeout: () => {}
  };
  globalThis.fetch = async () => new Response('{"error":"Room not found"}', {
    status: 404,
    headers: { "Content-Type": "application/json" }
  });

  try {
    connectRoomSocket("https://rooms.example", "AB12", "Guest", "access", "", "guest", {});
    await new Promise(resolve => setImmediate(resolve));
    assert.ok(drainRoomEvents().some(event => event.event === "room-closed"));
    assert.equal(retryScheduled, false);
  } finally {
    disconnectRoomSocket();
    globalThis.window = originalWindow;
    globalThis.fetch = originalFetch;
  }
});
