import assert from "node:assert/strict";
import test from "node:test";
import { roomRequest } from "../src/browser.js";

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
      "/auth/session",
      "POST",
      '{"passcode":"test"}',
      "",
      ""
    );
    await roomRequest("https://rooms.example", "/parties/ABCD", "GET", "{}", "", "");

    assert.equal(requests[0].options.body, '{"passcode":"test"}');
    assert.equal("body" in requests[1].options, false);
  } finally {
    globalThis.fetch = originalFetch;
  }
});
