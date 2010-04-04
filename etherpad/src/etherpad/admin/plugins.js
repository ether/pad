/**
 * Copyright 2009 RedHog, Egil Möller <egil.moller@piratpartiet.se>
 * 
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 * 
 *      http://www.apache.org/licenses/LICENSE-2.0
 * 
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS-IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

import("faststatic");
import("dispatch.{Dispatcher,PrefixMatcher,forward}");

import("etherpad.utils.*");
import("etherpad.collab.server_utils");
import("etherpad.globals.*");
import("etherpad.log");
import("etherpad.pad.padusers");
import("etherpad.pro.pro_utils");
import("etherpad.helpers");
import("etherpad.pro.pro_accounts.getSessionProAccount");
import("sqlbase.sqlbase");
import("sqlbase.sqlcommon");
import("sqlbase.sqlobj");
import("exceptionutils");
import("execution");

jimport("java.io.File",
        "java.io.DataInputStream", 
        "java.io.FileInputStream",
        "java.lang.Byte",
        "java.io.FileReader",
        "java.io.BufferedReader",
        "net.appjet.oui.JarVirtualFile");

pluginsLoaded = false;
pluginModules = {};
plugins = {};
hooks = {};
clientHooks = {};

function loadAvailablePlugin(pluginName) {
  if (plugins[pluginName] != undefined)
    return plugins[pluginName];

  var pluginsDir = new Packages.java.io.File("src/plugins");

  var pluginFile = new Packages.java.io.File(pluginsDir, pluginName + '/main.js');
  if (pluginFile.exists()) {
    var pluginModulePath = pluginFile.getPath().replace(new RegExp("src/\(.*\)\.js"), "$1").replace("/", ".", "g");
    var importStmt = "import('" + pluginModulePath + "')";
    try {
      var res = execution.fancyAssEval(importStmt, "main;");
      res = new res.init();
      return res;
    } catch (e) {
      log.info({errorLoadingPlugin:exceptionutils.getStackTracePlain(e)});
    }
  }
  return null;
}

function loadAvailablePlugins() {
  var pluginsDir = new Packages.java.io.File("src/plugins");

  var pluginNames = pluginsDir.list();

  for (i = 0; i < pluginNames.length; i++) {
    var plugin = loadAvailablePlugin(pluginNames[i]);
    if (plugin != null)
	pluginModules[pluginNames[i]] = plugin
  }
}

function loadPluginHooks(pluginName) {
  function registerHookNames(hookSet, type) {
    return function (hook) {
      var row = {hook:hook, type:type, plugin:pluginName};
      if (hookSet[hook] == undefined) hookSet[hook] = [];
      hookSet[hook].push(row);
      return row;
    }
  }
  plugins[pluginName] = pluginModules[pluginName].hooks.map(registerHookNames(hooks, 'server'));
  if (pluginModules[pluginName].client != undefined && pluginModules[pluginName].client.hooks != undefined)
    plugins[pluginName] = plugins[pluginName].concat(pluginModules[pluginName].client.hooks.map(registerHookNames(clientHooks, 'client')));
}

function unloadPluginHooks(pluginName) {
  for (var hookSet in [hooks, clientHooks])
    for (var hookName in hookSet) {
      var hook = hookSet[hookName];
      for (i = hook.length - 1; i >= 0; i--)
	if (hook[i].plugin == pluginName)
	  hook.splice(i, 1);
    }
  delete plugins[pluginName];
}

function loadInstalledHooks() {
  var sql = '' +
   'select ' +
   ' hook.name as hook, ' +
   ' hook_type.name as type, ' +
   ' plugin.name as plugin, ' +
   ' plugin_hook.original_name as original ' +
   'from ' +
   ' plugin ' +
   ' left outer join plugin_hook on ' +
   '  plugin.id = plugin_hook.plugin_id ' +
   ' left outer join hook on ' +
   '  plugin_hook.hook_id = hook.id ' +
   ' left outer join hook_type on ' +
   '  hook.type_id = hook_type.id ' +
   'order by hook.name, plugin.name';

  var rows = sqlobj.executeRaw(sql, {});
  for (var i = 0; i < rows.length; i++) {
    var row = rows[i];	

    if (plugins[row.plugin] == undefined)
      plugins[row.plugin] = [];
    plugins[row.plugin].push(row);

    var hookSet;

    if (row.type == 'server')
      hookSet = hooks;
    else if (row.type == 'client')
      hookSet = clientHooks;

    if (hookSet[row.hook] == undefined)
      hookSet[row.hook] = [];
    if (row.hook != 'null')
      hookSet[row.hook].push(row);
  }
}

function selectOrInsert(table, columns) {
  var res = sqlobj.selectSingle(table, columns);
  if (res !== null)
    return res;
  sqlobj.insert(table, columns);
  return sqlobj.selectSingle(table, columns);
}

function saveInstalledHooks(pluginName) {
  var plugin = sqlobj.selectSingle('plugin', {name:pluginName});

  if (plugin !== null) {
    sqlobj.deleteRows('plugin_hook', {plugin_id:plugin.id});
    if (plugins[pluginName] === undefined)
      sqlobj.deleteRows('plugin', {name:pluginName});
  }

  if (plugins[pluginName] !== undefined) {
    if (plugin === null)
      plugin = selectOrInsert('plugin', {name:pluginName});

    for (var i = 0; i < plugins[pluginName].length; i++) {
      var row = plugins[pluginName][i];

      var hook_type = selectOrInsert('hook_type', {name:row.type});
      var hook = selectOrInsert('hook', {name:row.hook, type_id:hook_type.id});

      sqlobj.insert("plugin_hook", {plugin_id:plugin.id, hook_id:hook.id});
    }
  }
}


function loadPlugins(force) {
  if (pluginsLoaded && force == undefined) return;
  pluginsLoaded = true;
  loadAvailablePlugins();
  loadInstalledHooks();
}


/* User API */
function enablePlugin(pluginName) {
  loadPlugins();
  loadPluginHooks(pluginName);
  saveInstalledHooks(pluginName);
  try {
    pluginModules[pluginName].install();
  } catch (e) {
    unloadPluginHooks(pluginName);
    saveInstalledHooks(pluginName);
    throw e;
  }
}

function disablePlugin(pluginName) {
  loadPlugins();
  try {
    pluginModules[pluginName].uninstall();
  } catch (e) {
    log.info({errorUninstallingPlugin:exceptionutils.getStackTracePlain(e)});
  }
  unloadPluginHooks(pluginName);
  saveInstalledHooks(pluginName);
}

function registerClientHandlerJS() {
  loadPlugins();
  for (pluginName in plugins) {
    var plugin = pluginModules[pluginName];
    if (plugin.client !== undefined) {
      helpers.includeJs("plugins/" + pluginName + "/main.js");
      if (plugin.client.modules != undefined)
        for (j = 0; j < client.modules.length; j++)
          helpers.includeJs("plugins/" + pluginName + "/" + plugin.client.modules[j] + ".js");
    }
  }
  helpers.addClientVars({hooks:clientHooks});
  helpers.includeJs("plugins.js");
}

function callHook(hookName, args) {
  loadPlugins();
  if (hooks[hookName] === undefined)
    return [];
  var res = [];

  for (var i = 0; i < hooks[hookName].length; i++) {
    var plugin = hooks[hookName][i];
    var pluginRes = pluginModules[plugin.plugin][plugin.original || hookName](args);
    if (pluginRes != undefined && pluginRes != null)
      for (var j = 0; j < pluginRes.length; j++)
        res.push(pluginRes[j]); /* Don't use Array.concat as it flatterns arrays within the array */
  }
  return res;
}

function callHookStr(hookName, args, sep, pre, post) {
  if (sep == undefined) sep = '';
  if (pre == undefined) pre = '';
  if (post == undefined) post = '';
  return callHook(hookName, args).map(function (x) { return pre + x + post}).join(sep || "");
}
