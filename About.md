## Inspiration

Services on the internet are designed, you need to login with an account connected to your id.
This way it is not designed to allow transfer accounts.
NFTs are a digital way to own access to a service.
This access can also be transfered by changing the ownership of the NFT.
We created a openid connect provider which combines both worlds.

## What it does

NFT-login is a provider speaking the well known openid connect protocol, which is an open industry standard for authentication.
All Web or Mobile Services speaking openid can be connected to the service and
would allow login users owning a special nft.

## How we built it

There are two things needed to build this provider. The part for the services, which connect to the provider is a server backend speaking the openid connect protocol. The other part is a end user frontend which connects to the blockchains to verify the ownership of the nft.

## Challenges we ran into

The openid connect provider is just a part in a complex system.
For the enduser to use it, there must be a marketplace to get a nft.
Also the user should login into a service, so we had to host a service, that 
uses nft-login for authorization. In the end we had to develop next to the nft-login provider a marketplace (we used an existing and customized it) and a service (svelte oidc example, which we also customized).

## Accomplishments that we're proud of

We are proud that we deployed a erc721 smart contract, hosted a marketplace,
and provide the owners access to a service using the nft-login provider.

## What we learned

We learned to deploy smart contracts, host web3 software on github pages and
how complicated metamask signatures can be.

## What's next for OIDC NFT Login

As of today nft-login just proves a user is the holder of a nft.
We want to develop more customizable nft contracts, so we can add attributes to the nft, so we can provide claims like

* default or premium account
* end date of service

## "Try it out" links

* https://nft-login.github.io/nft-login-demo/
* https://github.com/nft-login/nft-login
* https://nft-login.github.io/nft-login-marketplace/
* https://heco-nft-login-demo.4everland.app/

## Test it

The workflow is as following. You need a nft. We built a nft marketplace for easily getting tokens at https://nft-login.github.io/nft-login-marketplace/#/.

Get your token then return to https://nft-login.github.io/nft-login-demo/ and login.

During the login you are redirected to our project nft-login.net .

### Okex Blockchain

For OKEx Chain use this link https://nft-login.github.io/nft-login-demo/okt/ .

Watch here how you can test the technology with https://oidcdebugger.com/ 

[![NFT Login](https://img.youtube.com/vi/FZpdX5LvDoY/0.jpg)](https://www.youtube.com/watch?v=FZpdX5LvDoY)

### Heco Blockchain

Here is a video of the login on the heco chain with a scene of the Hecoinfo explorer.

[![NFT Login](https://img.youtube.com/vi/-Zhz4o2dfaY/0.jpg)](https://www.youtube.com/watch?v=-Zhz4o2dfaY)

Visit the login demo page hosted on 4everland https://heco-nft-login-demo.4everland.app .

## Notice
We also built the project Early Access Game NFT which is independent but visible in the demo videos as we use it to mint the nfts.

## Built with

rust, openidconnect, web3, github-pages, rocket, heco, nft, okexchain