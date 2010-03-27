function init() {
  this.hooks = ['aceInitInnerdocbodyHead', 'aceGetFilterStack', 'aceCreateDomLine'];
  this.aceInitInnerdocbodyHead = aceInitInnerdocbodyHead;
  this.aceGetFilterStack = aceGetFilterStack;
  this.aceCreateDomLine = aceCreateDomLine;
}

function aceInitInnerdocbodyHead(args) {
  args.iframeHTML.push('\'<link rel="stylesheet" type="text/css" href="/static/css/plugins/twitterStyleTags/pad.css"/>\'');
}

function aceGetFilterStack(args) {
  return [
    args.linestylefilter.getRegexpFilter(
      new RegExp("#[^,#=!\\s][^,#=!\\s]*", "g"), 'padtag'),
    args.linestylefilter.getRegexpFilter(
      new RegExp("=[^#=\\s][^#=\\s]*", "g"), 'padtagsearch')
  ];
}

function aceCreateDomLine(args) {
  if (args.cls.indexOf('padtagsearch') >= 0) {
    var href;
    cls = args.cls.replace(/(^| )padtagsearch:(\S+)/g, function(x0, space, padtagsearch) {
      href = '/ep/tag/?query=' + padtagsearch.substring(1);
      return space + "padtagsearch padtagsearch_" + padtagsearch.substring(1);
    });

   return [{
     cls: cls,
     extraOpenTags: '<a href="' + href.replace(/\"/g, '&quot;') + '">',
     extraCloseTags: '</a>'}];
  } else if (args.cls.indexOf('padtag') >= 0) {
    var href;
    cls = args.cls.replace(/(^| )padtag:(\S+)/g, function(x0, space, padtag) {
      href = '/ep/tag/?query=' + padtag.substring(1);
      return space + "padtag padtag_" + padtag.substring(1);
    });

   return [{
     cls: cls,
     extraOpenTags: '<a href="' + href.replace(/\"/g, '&quot;') + '">',
     extraCloseTags: '</a>'}];
  }
}

/* used on the client side only */
twitterStyleTags = new init();
