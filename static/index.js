window.addEventListener("load", function () {
  if (typeof window.ethereum !== "undefined") {
    console.log(
      "Web3 Detected! ",
      ethereum.networkVersion,
      parseInt(ethereum.chainId)
    );
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
  console.log(chainId);
  const urlParams = new URLSearchParams(queryString);
  urlParams.set("account", encodeURIComponent(
    account
  ));
  urlParams.set("chain_id", chainId);
  urlParams.set("signature", encodeURIComponent(signature));


  const query = `${urlParams.toString()}`;
  console.log(query);
  window.open("/authorize?" + query, '_self');
}

function chainDescription(chain) {
  var name = chain || `${parseInt(ethereum.chainId)}`;
  return (
    "Log in on " +
    name +
    " chain using your crypto account - You have to sign a message"
  );
}

function nftDescription(nft) {
  return nft ? "contract: " + nft + "" : "";
}

var sign_message_button = document.getElementById("sign_message_button");
sign_message_button.addEventListener("click", sign_message);

const queryString = window.location.search;
const chainId = parseInt(ethereum.chainId);
console.log(chainId);
console.log(queryString);
const urlParams = new URLSearchParams(queryString);
const nonce = urlParams.get("nonce");
const redirect_uri = urlParams.get("redirect_uri");
const chain = urlParams.get("chain");
const nft = urlParams.get("contract") || urlParams.get("client_id");
console.log(nonce);

document.getElementById("chain-description").innerHTML =
  chainDescription(chain);
document.getElementById("nft-description").innerHTML = nftDescription(nft);
