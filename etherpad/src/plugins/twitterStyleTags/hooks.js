import("etherpad.log");
import("dispatch.{Dispatcher,PrefixMatcher,forward}");
import("plugins.twitterStyleTags.controllers.tagBrowser");
import("sqlbase.sqlobj");

function handlePath() {
  return [[PrefixMatcher('/ep/tags/'), forward(tagBrowser)]];
}

function padModelWriteToDB(args) {
  /* Update tags for the pad */

  var new_tags = args.pad.text().match(new RegExp("#[^,#!\\s][^,#!\\s]*", "g"));
  if (new_tags == null) new_tags = new Array();
  for (i = 0; i < new_tags.length; i++)
    new_tags[i] = new_tags[i].substring(1);
  var new_tags_str = new_tags.join('#')

  var old_tags_row = sqlobj.selectSingle("PAD_TAG_CACHE", { id: args.padId });
  var old_tags_str;
  if (old_tags_row !== null)
    old_tags_str = old_tags_row['TAGS'];
  else
    old_tags_str = '';

  var old_tags = old_tags_str != '' ? old_tags_str.split('#') : new Array();

  if (new_tags_str != old_tags_str) {
    log.info({message: 'Updating tags', new_tags:new_tags, old_tags:old_tags});

    if (old_tags_row)
      sqlobj.update("PAD_TAG_CACHE", { id: args.padId }, {tags: new_tags.join('#')});
    else
      sqlobj.insert("PAD_TAG_CACHE", {id: args.padId, tags: new_tags.join('#')});

    sqlobj.deleteRows("PAD_TAG", {pad_id: args.padId});

    for (i = 0; i < new_tags.length; i++) {
      var tag_row = sqlobj.selectSingle("TAG", { name: new_tags[i] });
      if (tag_row === null) {
	sqlobj.insert("TAG", {name: new_tags[i]});
	tag_row = sqlobj.selectSingle("TAG", { name: new_tags[i] });
      }
      sqlobj.insert("PAD_TAG", {pad_id: args.padId, tag_id: tag_row['ID']});
    }
  }
}