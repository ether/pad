function callHook(hookName, args) {
  if (clientVars.hooks[hookName] === undefined)
    return [];
  var res = [];
  for (i = 0; i < clientVars.hooks[hookName].length; i++) {
    var plugin = clientVars.hooks[hookName][i];
    var pluginRes = eval(plugin.plugin)[plugin.original || hookName](args);
    if (pluginRes != undefined && pluginRes != null)
      res = res.concat(pluginRes);
  }
  return res;
}

function callHookStr(hookName, args, sep, pre, post) {
  if (sep == undefined) sep = '';
  if (pre == undefined) pre = '';
  if (post == undefined) post = '';
  return callHook(hookName, args).map(function (x) { return pre + x + post}).join(sep || "");
}
