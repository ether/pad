/**
 * Copyright 2009 RedHog, Egil Möller <egil.moller@piratpartiet.se>
 * Copyright 2010 Pita, Peter Martischka <petermartischka@googlemail.com>
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
import("etherpad.pro.domains");
import("etherpad.helpers");
import("etherpad.pro.pro_accounts.getSessionProAccount");
import("sqlbase.sqlbase");
import("sqlbase.sqlcommon");
import("sqlbase.sqlobj");
import("etherpad.pad.padutils");
import("etherpad.admin.plugins");
import("fastJSON");


function onRequest() {
  var type = "pads";
  if (request.params.type != undefined)
    type = request.params.type;
  var format = "html";
  if (request.params.format != undefined)
    format = request.params.format;
  var limit = 10;
  if (format == "sitemap")
    var limit = undefined;

  /* Make the query */
  var querySql = {sql:'PAD_META', 'params':[]};
  var hooks = plugins.callHook('queryToSql');
  for (i = 0; i < hooks.length; i++) {
    querySql = hooks[i](querySql);
  }

  /* Filter based on access privileges */
  var hooks = plugins.callHook('queryAccessSql');
  if (hooks.length == 0) {
    if (appjet.config.defaultAccess == 'none')
     querySql = {'sql':'(select ID from PAD_META where false)', 'params': []};
  } else {
    for (i = 0; i < hooks.length; i++) {
      querySql = hooks[i](querySql);
    }
  }

  /* Filter for the right domain */
  var domainSql = "ID NOT LIKE '%$%'";
  if (pro_utils.isProDomainRequest()) {
   domainSql = "ID LIKE '" + domains.getRequestDomainRecord().id + "$%'";
  }
  querySql.sql = "(select ID from " + querySql.sql + " as p where " + domainSql + ")";

  log.info(querySql);

  var clientVars = {
    userAgent: request.headers["User-Agent"],
    debugEnabled: request.params.djs,
    clientIp: request.clientAddr,
    colorPalette: COLOR_PALETTE,
    serverTimestamp: +(new Date),
  }

  var info = {
    bodyClass: 'nonpropad',
    config: appjet.config,
    padutils: padutils
  };

  var hooks = plugins.callHook('queryExtra');
  for (i = 0; i < hooks.length; i++) {
    hooks[i](querySql, info, clientVars);
  }

  var hooks = plugins.callHook('queryFormat');
  for (i = 0; i < hooks.length; i++) {
    if (hooks[i][type + '.' + format] != undefined) {
      return hooks[i][type + '.' + format](querySql, info, clientVars);
    }
  }
  if (request.params.type == undefined)
    for (i = 0; i < hooks.length; i++) {
      for (name in hooks[i]) {
	if (name.split('.', 2)[1] == format)
	  return hooks[i][name](querySql, info, clientVars);
      }
    }
  throw new Error("Unknown search output type/format combination type='" + type + "', format='" + format + "'");
}
