import("etherpad.log");
import("dispatch.{Dispatcher,PrefixMatcher,forward}");
import("sqlbase.sqlobj");
import("plugins.urlIndexer.controllers.urlBrowser");

function handlePath() {
  return [[PrefixMatcher('/ep/url'), forward(urlBrowser)]];
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