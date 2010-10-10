import("etherpad.log");
import("etherpad.utils");
import("dispatch.{Dispatcher,PrefixMatcher,forward}");
import("sqlbase.sqlobj");
import("etherpad.collab.server_utils");
import("etherpad.pad.padutils");

function padModelWriteToDB(args) {
  /* Update links for the pad */

  var new_links = args.pad.text().match(new RegExp("\\[\\[[^\\[\\]]*]]", "g"));
  if (new_links == null) new_links = new Array();
  for (i = 0; i < new_links.length; i++)
    new_links[i] = new_links[i].substring(2, new_links[i].length-2);
  var new_links_str = new_links.join(' ')

  var old_links_row = sqlobj.selectSingle("PAD_LINK_CACHE", { PAD_ID: args.padId });
  var old_links_str;
  if (old_links_row !== null)
    old_links_str = old_links_row['LINKS'];
  else
    old_links_str = '';

  if (new_links_str != old_links_str) {
    log.info({message: 'Updating links', new_links:new_links_str, old_links:old_links_str});

    if (old_links_row)
      sqlobj.update("PAD_LINK_CACHE", {PAD_ID: args.padId }, {LINKS: new_links.join(' ')});
    else
      sqlobj.insert("PAD_LINK_CACHE", {PAD_ID: args.padId, LINKS: new_links.join(' ')});

    sqlobj.deleteRows("PAD_LINK", {PAD_ID: args.padId});

    for (i = 0; i < new_links.length; i++) {
      sqlobj.insert("PAD_LINK", {PAD_ID: args.padId, LINK: new_links[i]});
    }
  }
}

function queryToSql(args) {
  return [function (querySql) {
    if (request.params.linksto == undefined || request.params.linksto == '') {
      return querySql;
    } else  {
      var padId = request.params.linksto;
      var padRev = null;
      if (padId.indexOf('/') >= 0) {
        padId = padId.split("/", 2);
	padRev = padId[1];
	padId = padId[0];
      }

      if (server_utils.isReadOnlyId(padId)) {
        readOnlyPadId = padId;
        padId = server_utils.readonlyToPadId(padId);
      } else {
        readOnlyPadId = server_utils.padIdToReadonly(padId);
      }

      if (padRev != null) {
        padId += '/' + padRev;
        readOnlyPadId += '/' + padRev;
      } else {
        padId += '%';
        readOnlyPadId += '%';
      }

      var sql = '' +
	'(select distinct subq.ID from ' +
	'  ' + querySql.sql + ' as subq ' +
	'  join PAD_LINK as u on ' +
	'   subq.ID =  u.PAD_ID and ' +
	'   u.LINK like ? or u.LINK like ? ) ';
       return {
	 'sql': sql,
	 'params': querySql.params.concat([padId, readOnlyPadId])
       };
    }
  }];
}

function querySummary() {
 return [utils.renderTemplateAsString("wikiStyleLinkQuerySummary.ejs", {}, ['wikiStyleLinks'])];
}

function queryRefiner() {
 return [utils.renderTemplateAsString("wikiStyleLinkQueryRefiner.ejs", {}, ['wikiStyleLinks'])];
}
