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

/* We need this to ensure that we only look at #public
   pads. */

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

/* So, maybe we want a similar sort of query?  Let's
   see...  I think we need SOME query, but it can be much
   simpler.

   So where is querySql currently coming from?  Answer: it
   is currently coming from the refine-by-tags mechanism.
   If we DIDN'T have a refine-by-tags mechanism, we would
   just want to look for pads containing a given URL!
   So let's try writing that query.*/

/* Will filter pads based on URLs  */

function urlFilterSql(querySql, url) {
log.info("Hi from urlFilterSql.");
  var sql = '' +
   '(select distinct subq.ID from ' +
   '  ' + querySql.sql + ' as subq ' +
   '  join PAD_URL as u on ' +
   '   subq.ID =  u.PAD_ID and ' +
   '   u.URL = ?) ';
  return {
   sql: sql,
   params: querySql.params.concat([url])
  };
}

/* And perhaps ideally we would have a box to search for
   URLs as part of the GUI, in case people don't know that
   they can just type the URL into the browser's location
   bar. */

/* This should just be a way to refine the query using
   tags.  I'm leaving it in for now, ALMOST unchanged from
   urlIndexer -- if we decide we don't want the
   refine-by-tag feature, we ought to be able to just
   delete this without any trouble. */

/* Or... maybe there is something else going on? */

/* If we're going to work with the tag idea (which we may
   as well!) then we should CONTINUE to insist that the
   page we've found include the #public tag.  (We could
   subsequently set some kind of "public-by-default?"
   flag that would make all pads public by default if
   true, but who knows if we'd ever want such a thing.) */

function onRequest() {  
  log.info("Hi from onRequest.");
  var urlOfInterest = request.params.query;
  var matchingPads = [];

  if(urlOfInterest !== undefined){
   /* A (sub)-query for #public pads */
   var querySql = tagQuery.getQueryToSql(['public'], []);

   querySql = urlFilterSql(querySql, urlOfInterest);
   log.info(querySql); 

   /* Limit is "how many entries to get back"; and 
      offset -- which is optional -- is
      "if there are more than that many, where do we start" */

   /* The reason for doing anything else with the pads is
      to be able to display some additional data to the
      user (not just a list of names).  */
   var matchingPadsSql = tagQuery.padInfoSql(querySql, 10);

   log.info(matchingPadsSql);

   matchingPads = sqlobj.executeRaw(matchingPadsSql.sql, matchingPadsSql.params);
  }
  else {
   urlOfInterest = "";
  }

  for (i = 0; i < matchingPads.length; i++) {
    matchingPads[i].TAGS = matchingPads[i].TAGS.split('#');
  }

  var isPro = pro_utils.isProDomainRequest();
  var userId = padusers.getUserId();

  /* Sets a bunch of javascript variables on the client side. 
     (What goes here is up to the client.)*/

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

  /* This is info that is set on to renderTemplateAsString -- these are just variables
     that are sent into the ejs.  */
  var info = {
    prefs: prefs,
    config: appjet.config,
    tagQuery: tagQuery,
    padIdToReadonly: server_utils.padIdToReadonly,
    tags: [],
    antiTags: [],
    newTags: [],
    matchingPads: matchingPads,
    urlOfInterest: urlOfInterest,
    matchingUrls: [],
    bodyClass: 'nonpropad',
    isPro: isPro,
    isProAccountHolder: isProUser,
    account: getSessionProAccount(), // may be falsy
  };

  var format = "html";
  if (request.params.format != undefined)
    format = request.params.format;

  /* Call renderHtml with the name of the plugin(s) where
     it should search for templates. */

  if (format == "html"){
    renderHtml("citationBrowser.ejs", info, ['findCitations', 'twitterStyleTags']);
}
  else if (format == "rss") {
    response.setContentType("application/xml; charset=utf-8");
    response.write(renderTemplateAsString("tagRss.ejs", info, 'findCitations'));
    if (request.acceptsGzip) {
      response.setGzip(true);
    }
  }
  return true;
}
