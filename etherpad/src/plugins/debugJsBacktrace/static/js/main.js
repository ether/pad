function debugJsBacktraceInit() {
  this.hooks = [];
}

/* used on the client side only */
debugJsBacktrace = new debugJsBacktraceInit();
