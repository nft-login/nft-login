window.addEventListener("load", function () {
  if (typeof window.ethereum !== "undefined") {
    console.log("Web3 Detected! " + ethereum.networkVersion);
  } else {
    console.log("No Web3 Detected... please install Metamask");
    document.getElementById("metamask_warning").hidden = false;
  }
});

async function sign_message() {
  const accounts = await ethereum.request({ method: "eth_requestAccounts" });
  const account = accounts[0];

  const payload = {
    account,
    nonce,
  };
  const message = account + nonce;

  var signature = await ethereum.request({
    method: "personal_sign",
    params: [message, account],
  });
  console.log(payload);
  console.log(signature);
  const query = queryString + "&account=" + encodeURIComponent(account) + "&signature=" + encodeURIComponent(signature);
  location.href = "authorize" + query;
}

var sign_message_button = document.getElementById("sign_message_button");
sign_message_button.addEventListener("click", sign_message);

const queryString = window.location.search;
console.log(queryString);
const urlParams = new URLSearchParams(queryString);
const nonce = urlParams.get("nonce");
const redirect_uri = urlParams.get("redirect_uri");
console.log(nonce);
