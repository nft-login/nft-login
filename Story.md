# Story

Here is a short story about this project.

## About us

We are a small team of software developers with a lot of interest in new technologies.

## Why did we build this project?

The hype around NFTs is on everyone's lips. However, their added value is not always clear to us. We wanted to make sure that NFTs don't just serve an end in themselves, but offer real added value to the owner.
To this end, the owner must be able to do something with his NFTs. The owner should get exclusive access to certain services.

## How we build it

Access to explicit services is nowadays possible via a personalized login. A common standard is openid connect. So we asked ourselves, can't we combine new technologies with this proven technology?
The fascinating thing about this connection is that the exclusive access is not linked to an individual, but to the possession of a credential in the form of an NFT.
To realize this, we have developed an OpenId Connect provider, which handles access via validation of the owner of an NFT token.
For a service provider this means that he only has to configure his openid connect client in a way that the login runs via our provider and he can grant access to the owners of nft tokens, which he might have sold himself before. For NFT token owners it means that they only need to sign a verification with their wallet to prove that they are the owner of an nft token and they will be granted access to the exclusive service.

## Conclusion

An entitlement to a service governed by NFT tokens can be transferred by the owner at will.
This opens up completely new business opportunities for content providers and many new access options for users, both temporarily and qualitatively.
