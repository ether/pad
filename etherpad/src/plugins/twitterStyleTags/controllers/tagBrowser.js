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

import("plugins.twitterStyleTags.models.tagQuery");

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
import("etherpad.pad.padutils");


function onRequest() {  
  var tags = new Array();
  var antiTags = new Array();

  if (request.params.query != undefined && request.params.query != '') {
    var query = request.params.query.split(',');
    for (i = 0; i < query.length; i++)
     if (query[i][0] == '!')
      antiTags.push(query[i].substring(1));
     else
      tags.push(query[i]);
  }

  /* Create the pad filter sql */
  var querySql = tagQuery.getQueryToSql(tags.concat(['public']), antiTags);

  /* Use the pad filter sql to figure out which tags to show in the tag browser this time. */
  var queryNewTagsSql = tagQuery.newTagsSql(querySql);
  var newTags = sqlobj.executeRaw(queryNewTagsSql.sql, queryNewTagsSql.params);

  /* Select the 10 last changed matching pads and some extra information on them. Except the Pro Pads*/ 
  var sql = '' +
    'select ' +
    '  m.id as ID, ' +
    '  DATE_FORMAT(m.lastWriteTime, \'%a, %d %b %Y %H:%i:%s GMT\') as lastWriteTime, ' +
    '  c.TAGS ' +
    'from ' +
       querySql.sql + ' as q ' +
    '  join PAD_SQLMETA as m on ' +
    '    m.id = q.ID ' +
    '  join PAD_TAG_CACHE as c on ' +
    '    c.PAD_ID = q.ID ' +
    'where ' +
    '  m.id NOT LIKE \'%$%\'' +
    'order by ' +
    '  m.lastWriteTime desc ' +
    'limit 10';
  var matchingPads = sqlobj.executeRaw(sql, querySql.params);

  for (i = 0; i < matchingPads.length; i++) {
    matchingPads[i].TAGS = matchingPads[i].TAGS.split('#');
  }

  var isPro = pro_utils.isProDomainRequest();
  var userId = padusers.getUserId();

  helpers.addClientVars({
   userAgent: request.headers["User-Agent"],
   debugEnabled: request.params.djs,
   clientIp: request.clientAddr,
   colorPalette: COLOR_PALETTE,
   serverTimestamp: +(new Date),
   isProPad: isPro,
   userIsGuest: padusers.isGuest(userId),
   userId: userId,
  });

  var isProUser = (isPro && ! padusers.isGuest(userId));


  padutils.setOptsAndCookiePrefs(request);
  var prefs = helpers.getClientVar('cookiePrefsToSet');
  var bodyClass = (prefs.isFullWidth ? "fullwidth" : "limwidth")

  var info = {
    prefs: prefs,
    config: appjet.config,
    tagQuery: tagQuery,
    padIdToReadonly: server_utils.padIdToReadonly,
    tags: tags,
    antiTags: antiTags,
    newTags: newTags,
    matchingPads: matchingPads,
    bodyClass: 'nonpropad',
    isPro: isPro,
    isProAccountHolder: isProUser,
    account: getSessionProAccount(), // may be falsy
  };

  var format = "html";
  if (request.params.format != undefined)
    format = request.params.format;

  if (format == "html")
    renderHtml("tagBrowser.ejs", info, 'twitterStyleTags');
  else if (format == "rss") {
    response.setContentType("application/xml; charset=utf-8");
    response.write(renderTemplateAsString("tagRss.ejs", info, 'twitterStyleTags'));
    if (request.acceptsGzip) {
      response.setGzip(true);
    }
  }
  return true;
}
