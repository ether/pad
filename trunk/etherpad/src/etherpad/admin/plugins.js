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

function loadAvailablePlugin(pluginName) {
  if (plugins[pluginName] != undefined)
    return plugins[pluginName];

  var pluginsDir = new Packages.java.io.File("src/plugins");

  var pluginFile = new Packages.java.io.File(pluginsDir, pluginName + '/main.js');
  if (pluginFile.exists()) {
    var pluginModulePath = pluginFile.getPath().replace(new RegExp("src/\(.*\)\.js"), "$1").replace("/", ".", "g");
    var importStmt = "import('" + pluginModulePath + "')";
    try {
      return execution.fancyAssEval(importStmt, "main;");
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

function loadInstalledHooks() {
  var sql = '' +
   'select ' +
   ' hook.name as hook, ' +
   ' plugin.name as plugin, ' +
   ' plugin_hook.original_name as original_hook ' +
   'from ' +
   ' plugin ' +
   ' left outer join plugin_hook on ' +
   '  plugin.id = plugin_hook.plugin_id ' +
   ' left outer join hook on ' +
   '  plugin_hook.hook_id = hook.id ' +
   'order by hook.name, plugin.name';

  var rows = sqlobj.executeRaw(sql, {});
  for (var i = 0; i < rows.length; i++) {
    if (hooks[rows[i].hook] == undefined)
      hooks[rows[i].hook] = [];0
    if (plugins[rows[i].plugin] == undefined)
      plugins[rows[i].plugin] = [];
    plugins[rows[i].plugin].push({hookName:rows[i].hook, originalHook:rows[i].originalName});
    if (rows[i].hook != 'null')
      hooks[rows[i].hook].push({pluginName:rows[i].plugin, originalHook:rows[i].originalName});
  }
}

function loadPlugins() {
  if (pluginsLoaded) return;
  pluginsLoaded = true;
  loadAvailablePlugins();
  loadInstalledHooks();
}

function registerHook(pluginName, hookName, originalHook) {
  if (originalHook == undefined) originalHook = null;
  plugins[pluginName].push({hookName:hookName, originalHook:originalHook});
  if (hooks[hookName] === undefined) hooks[hookName] = [];
  hooks[hookName].push({pluginName:pluginName, originalHook:originalHook});

  var plugin = sqlobj.selectSingle('plugin', {name:pluginName});
  var hook = sqlobj.selectSingle('hook', {name:hookName});
  if (hook == null) {
    sqlobj.insert('hook', {name:hookName});
    hook = sqlobj.selectSingle('hook', {name:hookName});
  }
  sqlobj.insert("plugin_hook", {plugin_id:plugin.id, hook_id:hook.id, original_name:originalHook});
}

function unregisterHooks(pluginName) {
  delete plugins[pluginName];

  for (hookName in hooks) {
    hooks[hookName] = hooks[hookName].filter(function (plugin) { return plugin.pluginName != pluginName; });
    if (hooks[hookName].length == 0)
      delete hooks[hookName];
  }

  var plugin = sqlobj.selectSingle('plugin', {name:pluginName});
  if (plugin != undefined)
    sqlobj.deleteRows('plugin_hook', {plugin_id:plugin.id});
}

/* User API */
function enablePlugin(pluginName) {
  loadPlugins();
  if (pluginModules[pluginName] === undefined)
    throw new Error ("Unable to find a plugin named " + pluginName);
  if (plugins[pluginName] !== undefined)
    throw new Error ("Atempting to reenable the already enabled plugin " + pluginName);
  sqlobj.insert("plugin", {name:pluginName});
  plugins[pluginName] = [];
  var pluginHooks = pluginModules[pluginName].hooks || [];
  pluginHooks = pluginHooks.concat(pluginModules[pluginName].hooksShared || []);
  for (var i = 0; i < pluginHooks.length; i++)
    registerHook(pluginName, pluginHooks[i]);
  pluginModules[pluginName].install();
}

function disablePlugin(pluginName) {
  loadPlugins();
  try 
  {
    pluginModules[pluginName].uninstall();
  } catch (e) {
    log.info({errorUninstallingPlugin:exceptionutils.getStackTracePlain(e)});
  }
  unregisterHooks(pluginName);
  delete plugins[pluginName];
  sqlobj.deleteRows("plugin", {name:pluginName});
}

function callHook(hookName, args) {
  loadPlugins();
  if (hooks[hookName] === undefined)
    return [];
  var res = [];
  for (i = 0; i < hooks[hookName].length; i++) {
    var plugin = hooks[hookName][i];
    var pluginRes = pluginModules[plugin.pluginName][plugin.originalHook || hookName](args);
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
