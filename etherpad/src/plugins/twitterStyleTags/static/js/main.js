function init() {
 this.hooks = ['aceGetFilterStack'];
 this.aceGetFilterStack = aceGetFilterStack;
}

function aceGetFilterStack() {
 console.log('aceGetFilterStack');
}

/* used on the client side only */
twitterStyleTags = new init();
