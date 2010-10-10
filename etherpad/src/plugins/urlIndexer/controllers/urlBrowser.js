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
import("etherpad.admin.plugins");
import("etherpad.pad.padutils");

function urlSql(querySql, limit, offset) {
  var sql = '' +
   'select ' +
   '  u.URL, ' +
   '  m.id as ID, ' +
   '  DATE_FORMAT(m.lastWriteTime, \'%a, %d %b %Y %H:%i:%s GMT\') as lastWriteTime, ' +
   '  c.TAGS ' +
   'from ' +
      querySql.sql + ' as q ' +
   '  join PAD_SQLMETA as m on ' +
   '    m.id = q.ID ' +
   '  join PAD_TAG_CACHE as c on ' +
   '    c.PAD_ID = q.ID ' +
   '  join PAD_URL as u on ' +
   '    u.PAD_ID = q.ID ' +
   'where ' +
   '  m.id NOT LIKE \'%$%\'' +
   'order by ' +
   '  u.URL asc ';
  if (limit != undefined)
   sql += 'limit ' + limit + " ";
  if (offset != undefined)
   sql += 'offset ' + offset + " ";
  return {
   sql: sql,
   params: querySql.params
  };
}

function onRequest() {  
  var tags = tagQuery.queryToTags(request.params.query);

  /* Create the pad filter sql */
  var querySql = tagQuery.getQueryToSql(tags.tags.concat(['public']), tags.antiTags);

  var hooks = plugins.callHook('queryToSql');
  for (i = 0; i < hooks.length; i++) {
    querySql = hooks[i](querySql);
  }

  /* Use the pad filter sql to figure out which tags to show in the tag browser this time. */
  var queryNewTagsSql = tagQuery.newTagsSql(querySql);
  var newTags = sqlobj.executeRaw(queryNewTagsSql.sql, queryNewTagsSql.params);

  url = urlSql(querySql, 10);
  var matchingUrls = sqlobj.executeRaw(url.sql, url.params);

  for (i = 0; i < matchingUrls.length; i++) {
    matchingUrls[i].TAGS = matchingUrls[i].TAGS.split('#');
  }

  helpers.addClientVars({
   userAgent: request.headers["User-Agent"],
   debugEnabled: request.params.djs,
   clientIp: request.clientAddr,
   colorPalette: COLOR_PALETTE,
   serverTimestamp: +(new Date),
  });

  var info = {
    config: appjet.config,
    tagQuery: tagQuery,
    padIdToReadonly: server_utils.padIdToReadonly,
    tags: tags.tags,
    antiTags: tags.antiTags,
    newTags: newTags,
    matchingPads: [],
    matchingUrls: matchingUrls,
    bodyClass: 'nonpropad',
  };

  var format = "html";
  if (request.params.format != undefined)
    format = request.params.format;

  if (format == "html")
    renderHtml("urlBrowser.ejs", info, ['urlIndexer', 'twitterStyleTags']);
  else if (format == "rss") {
    response.setContentType("application/xml; charset=utf-8");
    response.write(renderTemplateAsString("tagRss.ejs", info, ['urlIndexer']));
    if (request.acceptsGzip) {
      response.setGzip(true);
    }
  }
  return true;
}
