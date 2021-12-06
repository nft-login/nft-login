const express = require("express");
const { auth } = require("express-openid-connect");

const app = express();

app.use(
  auth({
    issuerBaseURL: "https://nft-login.net/heco/authorize/",
    baseURL: "http://localhost:3000",
    clientID: "0xa0d4E5CdD89330ef9d0d1071247909882f0562eA",
    clientSecret: "SECRET",
    secret: "LONG_RANDOM_STRING",
    idpLogout: false,
    authorizationParams: {
      response_type: "code id_token",
      scope: "openid profile",
    },
  })
);

app.get("/", function (req, res) {
  const user = req.oidc.user;
  if (!user) {
    return res.redirect("/login");
  }
  return res.json(user);
});

app.listen(3000, () => {
  console.log("Server is running on http://localhost:3000");
});
