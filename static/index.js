window.addEventListener("load", function () {
  if (typeof window.ethereum !== "undefined") {
    console.log("Web3 Detected! " + ethereum.networkVersion);
  } else {
    console.log("No Web3 Detected... please install Metamask");
    document.getElementById("metamask_warning").hidden = false;
  }
});

async function sign_message() {
  var message = "just random";
  const accounts = await ethereum.request({ method: "eth_requestAccounts" });
  const account = accounts[0];
  var signature = await ethereum.request({
    method: "personal_sign",
    params: [message, account],
  });
  console.log(signature);
}

var sign_message_button = document.getElementById("sign_message_button");
sign_message_button.addEventListener("click", sign_message);
