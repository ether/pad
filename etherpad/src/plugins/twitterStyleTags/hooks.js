import("etherpad.log");
import("dispatch.{Dispatcher,PrefixMatcher,forward}");
import("plugins.twitterStyleTags.models.tagQuery");
import("sqlbase.sqlobj");
import("etherpad.collab.server_utils");
import("etherpad.utils");
import("etherpad.pad.padutils");
import("fastJSON");

function padModelWriteToDB(args) {
  /* Update tags for the pad */

  var new_tags = args.pad.text().match(new RegExp("#[^,#=!\\s][^,#=!\\s]*", "g"));
  if (new_tags == null) new_tags = new Array();
  for (i = 0; i < new_tags.length; i++)
    new_tags[i] = new_tags[i].substring(1);
  var new_tags_str = new_tags.join('#')

  var old_tags_row = sqlobj.selectSingle("PAD_TAG_CACHE", { PAD_ID: args.padId });
  var old_tags_str;
  if (old_tags_row !== null)
    old_tags_str = old_tags_row['TAGS'];
  else
    old_tags_str = '';

  // var old_tags = old_tags_str != '' ? old_tags_str.split('#') : new Array();

  if (new_tags_str != old_tags_str) {
    // log.info({message: 'Updating tags', new_tags:new_tags, old_tags:old_tags});

    if (old_tags_row)
      sqlobj.update("PAD_TAG_CACHE", {PAD_ID: args.padId }, {TAGS: new_tags.join('#')});
    else
      sqlobj.insert("PAD_TAG_CACHE", {PAD_ID: args.padId, TAGS: new_tags.join('#')});

    sqlobj.deleteRows("PAD_TAG", {PAD_ID: args.padId});

    for (i = 0; i < new_tags.length; i++) {
      var tag_row = sqlobj.selectSingle("TAG", { NAME: new_tags[i] });
      if (tag_row === null) {
	sqlobj.insert("TAG", {NAME: new_tags[i]});
	tag_row = sqlobj.selectSingle("TAG", { NAME: new_tags[i] });
      }
      sqlobj.insert("PAD_TAG", {PAD_ID: args.padId, TAG_ID: tag_row['ID']});
    }
  }
}

function queryAccessSql(args) {
  return [function (querySql) {
    return tagQuery.getQueryToSql(['public'], [], querySql);
  }];
}

function queryToSql(args) {
  return [function (querySql) {
    if (request.params.query == undefined || request.params.query == '') {
      return querySql;
    } else  {
      var tags = tagQuery.queryToTags(request.params.query);

      return tagQuery.getQueryToSql(tags.tags, tags.antiTags, querySql);
    }
  }];
}

function queryExtra() {
  return [function (querySql, info, clientVars) {
    var tags = tagQuery.queryToTags(request.params.query);

    var queryNewTagsSql = tagQuery.newTagsSql(querySql);
    var newTags = sqlobj.executeRaw(queryNewTagsSql.sql, queryNewTagsSql.params);

    info.tagQuery = tagQuery;
    info.tags = tags.tags;
    info.antiTags = tags.antiTags;
    info.newTags = newTags;
    info.padIdToReadonly = server_utils.padIdToReadonly;
  }];
}

function queryFormat() {
  function createFormat(format) {
    return function (querySql, info, clientVars) {
      var tags = tagQuery.queryToTags(request.params.query);

      var limit = 10;
      if (format == 'sitemap')
	limit = undefined;

      padSql = tagQuery.padInfoSql(querySql, limit);
      var matchingPads = sqlobj.executeRaw(padSql.sql, padSql.params);

      for (i = 0; i < matchingPads.length; i++) {
	matchingPads[i].TAGS = matchingPads[i].TAGS.split('#');
      }
      
      info.matchingPads = matchingPads;

      if (format == "html") {
	utils.renderHtml("tagBrowser.ejs", info, ['twitterStyleTags', 'search']);
      } else if (format == "rss") {
	response.setContentType("application/xml; charset=utf-8");
	response.write(utils.renderTemplateAsString("tagRss.ejs", info, ['twitterStyleTags', 'search']));
	if (request.acceptsGzip) {
	  response.setGzip(true);
	}
      } else if (format == "sitemap") {
	response.setContentType("application/xml; charset=utf-8");
	response.write(utils.renderTemplateAsString("tagSitemap.ejs", info, ['twitterStyleTags', 'search']));
	if (request.acceptsGzip) {
	  response.setGzip(true);
	}
      } else if (format == "json") {
	response.setContentType("application/json; charset=utf-8");
	response.write(fastJSON.stringify(info));
	if (request.acceptsGzip) {
	  response.setGzip(true);
	}
      } else {
        throw new Error("Unknown format " + format);
      }
      return true;
    };
  }

  return [{'pads.html': createFormat('html'),
           'pads.rss': createFormat('rss'),
           'pads.sitemap': createFormat('sitemap'),
           'pads.json': createFormat('json')
         }];
}

function querySummary(args) {
  var res = args.template.include("twitterStyleTagsQuerySummary.ejs", {}, ['twitterStyleTags']);
  if (res.replace(new RegExp("^[ \n]*"), "").replace(new RegExp("[ \n]*$"), "") == '')
    return [];
  return [res];
}

function queryRefiner(args) {
 return [args.template.include("twitterStyleTagsQueryRefiner.ejs", {}, ['twitterStyleTags'])];
}

function docbarItemsSearch() {
 return ["<td class='docbarbutton'><a href='/ep/search?type=pads'>Pads</a></td>"];
}

function editBarItemsLeftPad(arg) {
  return [arg.template.include('twitterStyleTagsEditbarButtons.ejs', undefined, ['twitterStyleTags'])];
}
