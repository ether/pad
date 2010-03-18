function init() {
 this.hooks = ['kafoo'];
 this.kafoo = kafoo;
}

function kafoo() {
 alert('hej');
}

/* used on the client side only */
testplugin = new init();
