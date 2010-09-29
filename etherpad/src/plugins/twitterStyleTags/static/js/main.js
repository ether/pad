function twitterStyleTagsInit() {
  this.hooks = ['aceInitInnerdocbodyHead', 'aceGetFilterStack', 'aceCreateDomLine'];
}

twitterStyleTagsInit.prototype.aceInitInnerdocbodyHead = function(args) {
  args.iframeHTML.push('\'<link rel="stylesheet" type="text/css" href="/static/css/plugins/twitterStyleTags/pad.css"/>\'');
}

twitterStyleTagsInit.prototype.aceGetFilterStack = function(args) {
  return [
    args.linestylefilter.getRegexpFilter(
      new RegExp("#[^,#=!\\s\"'][^,#=!\\s}\"']*", "g"), 'padtag'),
    args.linestylefilter.getRegexpFilter(
      new RegExp("=[^#=\\s\"'][^#=\\s\"']*", "g"), 'padtagsearch')
  ];
}

twitterStyleTagsInit.prototype.aceCreateDomLine = function(args) {
  if (args.cls.indexOf('padtagsearch') >= 0) {
    var href;
    cls = args.cls.replace(/(^| )padtagsearch:(\S+)/g, function(x0, space, padtagsearch) {
      var tag = encodeURI(padtagsearch.substring(1));
      href = '/ep/tag/?query=' + tag;
      return space + "padtagsearch padtagsearch_" + tag;
    });

   return [{
     cls: cls,
     extraOpenTags: '<a href="' + href.replace(/\"/g, '&quot;') + '">',
     extraCloseTags: '</a>'}];
  } else if (args.cls.indexOf('padtag') >= 0) {
    var href;
    cls = args.cls.replace(/(^| )padtag:(\S+)/g, function(x0, space, padtag) {
      var tag = encodeURI(padtag.substring(1));
      href = '/ep/tag/?query=' + tag;
      return space + "padtag padtag_" + tag;
    });

   return [{
     cls: cls,
     extraOpenTags: '<a href="' + href.replace(/\"/g, '&quot;') + '">',
     extraCloseTags: '</a>'}];
  }
}

/* used on the client side only */
twitterStyleTags = new twitterStyleTagsInit();
