function debugRaiseExceptionInit() {
  this.hooks = [];
  this.debugRaiseExceptionClicked = debugRaiseExceptionClicked;
}

function debugRaiseExceptionClicked () {
    throw "Test exception";
}

/* used on the client side only */
debugRaiseException = new debugRaiseExceptionInit();
