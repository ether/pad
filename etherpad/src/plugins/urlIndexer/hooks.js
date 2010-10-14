import("etherpad.log");
import("dispatch.{Dispatcher,PrefixMatcher,forward}");
import("sqlbase.sqlobj");
import("etherpad.helpers");
import("etherpad.collab.server_utils");
import("etherpad.utils.*");

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

REGEX_WORDCHAR = /[\u0030-\u0039\u0041-\u005A\u0061-\u007A\u00C0-\u00D6\u00D8-\u00F6\u00F8-\u00FF\u0100-\u1FFF\u3040-\u9FFF\uF900-\uFDFF\uFE70-\uFEFE\uFF10-\uFF19\uFF21-\uFF3A\uFF41-\uFF5A\uFF66-\uFFDC]/;
REGEX_URLCHAR = new RegExp('('+/[-:@a-zA-Z0-9_.,~%+\/\\?=&#;()$]/.source+'|'+REGEX_WORDCHAR.source+')');
REGEX_URL = new RegExp(/(?:(?:https?|s?ftp|ftps|file|smb|afp|nfs|(x-)?man|gopher|txmt):\/\/|mailto:)/.source+REGEX_URLCHAR.source+'*(?![:.,;])'+REGEX_URLCHAR.source, 'g');

function padModelWriteToDB(args) {
  /* Update tags for the pad */

  var new_urls = args.pad.text().match(REGEX_URL);
  if (new_urls == null) new_urls = new Array();
  var new_urls_str = new_urls.join(' ')

  var old_urls_row = sqlobj.selectSingle("PAD_URL_CACHE", { PAD_ID: args.padId });
  var old_urls_str;
  if (old_urls_row !== null)
    old_urls_str = old_urls_row['URLS'];
  else
    old_urls_str = '';

  // var old_urls = old_urls_str != '' ? old_urls_str.split(' ') : new Array();

  if (new_urls_str != old_urls_str) {
    // log.info({message: 'Updating urls', new_urls:new_urls, old_urls:old_urls});

    if (old_urls_row)
      sqlobj.update("PAD_URL_CACHE", {PAD_ID: args.padId }, {URLS: new_urls.join(' ')});
    else
      sqlobj.insert("PAD_URL_CACHE", {PAD_ID: args.padId, URLS: new_urls.join(' ')});

    sqlobj.deleteRows("PAD_URL", {PAD_ID: args.padId});

    for (i = 0; i < new_urls.length; i++) {
      sqlobj.insert("PAD_URL", {PAD_ID: args.padId, URL: new_urls[i]});
    }
  }
}

function queryFormat() {
 return [{'urls.html': function (querySql, info, clientVars) {
   url = urlSql(querySql, 10);
   var matchingUrls = sqlobj.executeRaw(url.sql, url.params);

   for (i = 0; i < matchingUrls.length; i++) {
     if (matchingUrls[i].TAGS != undefined) {
       matchingUrls[i].TAGS = matchingUrls[i].TAGS.split('#');
     }
   }

   helpers.addClientVars(clientVars);

   //info.tagQuery. = tagQuery;
   info.padIdToReadonly = server_utils.padIdToReadonly;
   info.matchingPads = [];
   info.matchingUrls = matchingUrls;

   renderHtml("urlBrowser.ejs", info, ['urlIndexer', 'search']);
   return true;
 }}];
}


function docbarItemsSearch() {
 return ["<td class='docbarbutton'><a href='/ep/search?type=urls'>URLs</a></td>"];
}

