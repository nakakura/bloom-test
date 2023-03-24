const Peer = window.Peer;

(async function main() {
  const localId = document.getElementById("js-local-id");
  const localText = document.getElementById("js-local-text");
  const connectTrigger = document.getElementById("js-connect-trigger");
  const closeTrigger = document.getElementById("js-close-trigger");
  const sendTrigger = document.getElementById("js-send-trigger");
  const remoteId = document.getElementById("js-remote-id");
  const messages = document.getElementById("js-messages");
  const meta = document.getElementById("js-meta");
  const sdkSrc = document.querySelector("script[src*=skyway]");

  meta.innerText = `
    UA: ${navigator.userAgent}
    SDK: ${sdkSrc ? sdkSrc.src : "unknown"}
  `.trim();

  let api_key = window.__SKYWAY_KEY__;

  // ../_shared/key.jsに記載せずパスから取る場合
  const queryString = window.location.search;
  const urlParams = new URLSearchParams(queryString);
  const key = urlParams.get("api_key");
  if (key) api_key = key;

  // peer_idはnew Peerの第一引数で指定しなければランダムで割り当てられるが、
  // 毎回ROS側で変更が手間であればここで設定する。
  const peer_id = "target_id";
  const peer = (window.peer = new Peer(peer_id, {
    key: api_key,
    debug: 3,
  }));

  // Register connecter handler
  connectTrigger.addEventListener("click", () => {
    // Note that you need to ensure the peer has connected to signaling server
    // before using methods of peer instance.
    if (!peer.open) {
      return;
    }

    const dataConnection = peer.connect(remoteId.value, {
      serialization: "none",
      metadata: {
        connection_id: "string_connections",
      },
    });

    dataConnection.once("open", async () => {
      messages.textContent += `=== DataConnection has been opened ===\n`;

      sendTrigger.addEventListener("click", onClickSend);
    });

    dataConnection.on("data", (data) => {
      const text_decoder = new TextDecoder("utf-8");
      const recv_text = text_decoder.decode(data);
      messages.textContent += `Remote: ${recv_text}\n`;
    });

    dataConnection.once("close", () => {
      messages.textContent += `=== DataConnection has been closed ===\n`;
      sendTrigger.removeEventListener("click", onClickSend);
    });

    // Register closing handler
    closeTrigger.addEventListener("click", () => dataConnection.close(true), {
      once: true,
    });

    function onClickSend() {
      const data = localText.value;
      dataConnection.send(data);

      messages.textContent += `You: ${data}\n`;
      localText.value = "";
    }
  });

  peer.once("open", (id) => (localId.textContent = id));

  // Register connected peer handler
  peer.on("connection", (dataConnection) => {
    dataConnection.once("open", async () => {
      messages.textContent += `=== DataConnection has been opened ===\n`;
      sendTrigger.addEventListener("click", onClickSend);
      console.log(dataConnection.metadata);
      console.log(dataConnection.serialization);
    });

    dataConnection.on("data", (data) => {
      const text_decoder = new TextDecoder("utf-8");
      const recv_text = text_decoder.decode(data);
      messages.textContent += `Remote: ${recv_text}\n`;
    });

    dataConnection.once("close", () => {
      messages.textContent += `=== DataConnection has been closed ===\n`;
      sendTrigger.removeEventListener("click", onClickSend);
    });

    // Register closing handler
    closeTrigger.addEventListener("click", () => dataConnection.close(true), {
      once: true,
    });

    function onClickSend() {
      const data = localText.value;
      dataConnection.send(data);

      messages.textContent += `You: ${data}\n`;
      localText.value = "";
    }
  });

  peer.on("error", console.error);
})();
