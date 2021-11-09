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
  const message = "" + account + ";" + encodeURIComponent(nonce);
  console.log(message);
  var signature = await ethereum.request({
    method: "personal_sign",
    params: [account, message],
  });
  console.log(payload);
  console.log(message);
  console.log(signature);
  const query =
    queryString +
    "&account=" +
    encodeURIComponent(account) +
    "&signature=" +
    encodeURIComponent(signature);
  location.href = "authorize" + query;
}

function chainDescription(chain) {
  var name = chain || "Kovan";
  return "Log in on " + name + " using your crypto account";
}

function nftDescription(nft) {
  return "" + nft + "";
}

var sign_message_button = document.getElementById("sign_message_button");
sign_message_button.addEventListener("click", sign_message);

const queryString = window.location.search;
console.log(queryString);
const urlParams = new URLSearchParams(queryString);
const nonce = urlParams.get("nonce");
const redirect_uri = urlParams.get("redirect_uri");
const chain = urlParams.get("chain");
const nft = urlParams.get("client_id");
console.log(nonce);

document.getElementById("chain-description").innerHTML =
  chainDescription(chain);
document.getElementById("nft-description").innerHTML =
  nftDescription(nft);
