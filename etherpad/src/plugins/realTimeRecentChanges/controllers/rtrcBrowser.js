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
import("fastJSON");

/* This is on the server side.  This is where
template.render etc. SEE helpers.includeJS renderhtml --

Right now I've got the onRequest from twitterStyleTags
copied and pasted in here, just changing it to make it
call rtrc browser.  Is this at all sensible?  Maybe.*/
function onRequest() {
  var tags = tagQuery.queryToTags(request.params.query);

  /* Create the pad filter sql */
  var querySql = tagQuery.getQueryToSql(tags.tags.concat(['public']), tags.antiTags);

  /* Use the pad filter sql to figure out which tags to show in the tag browser this time. */
  var queryNewTagsSql = tagQuery.newTagsSql(querySql);
  var newTags = sqlobj.executeRaw(queryNewTagsSql.sql, queryNewTagsSql.params);

  padSql = tagQuery.padInfoSql(querySql, 10);
  var matchingPads = sqlobj.executeRaw(padSql.sql, padSql.params);

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
    tags: tags.tags,
    antiTags: tags.antiTags,
    newTags: newTags,
    matchingPads: matchingPads
  };

  var format = "html";
  if (request.params.format != undefined)
    format = request.params.format;

  if (format == "json") {
    response.setContentType("application/json; charset=utf-8");
    response.write(fastJSON.stringify(info));
    if (request.acceptsGzip) {
      response.setGzip(true);
    }
  } else {
    info['tagQuery'] = tagQuery;
    info['prefs'] = prefs;
    info['config'] = appjet.config;
    info['padIdToReadonly'] = server_utils.padIdToReadonly;
    info['bodyClass'] = 'nonpropad';
    info['isPro'] = isPro;
    info['isProAccountHolder'] = isProUser;
    info['account'] = getSessionProAccount(); // may be falsy

    if (format == "html") {
      renderHtml("rtrcBrowser.ejs", info, ['realTimeRecentChanges']);
    } else if (format == "rss") {
      response.setContentType("application/xml; charset=utf-8");
      response.write(renderTemplateAsString("tagRss.ejs", info, ['twitterStyleTags']));
      if (request.acceptsGzip) {
	response.setGzip(true);
      }
   } else {
     throw new Error("Unknown format " + format);
   }
  }
  return true;
}
