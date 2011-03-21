function openingDesignInit() {
  this.hooks = ['aceInitInnerdocbodyHead', 'aceAttribsToClasses', 'aceCreateDomLine'];
  this.currentImage = {id: undefined, link: undefined};
}

openingDesignInit.prototype.aceInitInnerdocbodyHead = function(args) {
  args.iframeHTML.push('\'<link rel="stylesheet" type="text/css" href="/static/css/plugins/openingDesign/ace.css"/>\'');
  args.iframeHTML.push('\'\\x3cscript type="text/javascript" src="/static/js/jquery-1.3.2.js">\\x3c/script>\'');
  args.iframeHTML.push('\'\\x3cscript type="text/javascript" src="/static/js/plugins/openingDesign/ace_inner.js">\\x3c/script>\'');
};

openingDesignInit.prototype.aceAttribsToClasses = function(args) {
  if (args.key == 'openingDesignIsImage' && args.value != "")
    return ["openingDesignIsImage", "openingDesignImageId:" + args.value];
  else if (args.key.indexOf('openingDesignImageObject') == 0)
    return [args.key + ":" + args.value];
}

openingDesignInit.prototype.aceCreateDomLine = function(args) {
  if (args.cls.indexOf('openingDesignIsImage') >= 0) {
   var clss = [];
   var imageObjects = {};
   var imageId = undefined;
   $.each(args.cls.split(" "), function (i, cls) {
     var parts = cls.split(":");
     if (cls.indexOf("openingDesignImageId:") == 0) {
       imageId = parts[1];
     } if (cls.indexOf("openingDesignImageObject:") == 0) {
       imageObjects[parts[1]] = parts[2];
     } else {
       clss.push(cls);
     }
   });

   if (this.currentImage.id == imageId) {
     this.updateImageFromPad(imageObjects);
   }

   return [{cls: clss.join(" "), extraOpenTags: '<a class="openingDesignImageLink">', extraCloseTags: '</a>'}];
  }
}

openingDesignInit.prototype.updateImageFromPad = function(imageObjects) {
  console.log({updateImageFromPad:imageObjects});
}

openingDesignInit.prototype.selectImage = function(imageLink) {
  var imageId;
  $.each(imageLink.classList, function (idx, cls) {
    var parts = cls.split(":");
    if (parts[0] == "openingDesignImageId")
      imageId = parts[1];
  });

  console.log({imageId:imageId});
  this.currentImage = {'id': imageId, 'link': imageLink};
}

openingDesignInit.prototype.insertImage = function(event) {
  padeditor.ace.callWithAce(function (ace) {
    rep = ace.ace_getRep();

    ace.ace_replaceRange(rep.selStart, rep.selEnd, "I");
    ace.ace_performSelectionChange([rep.selStart[0],rep.selStart[1]-1], rep.selStart, false);
    ace.ace_performDocumentApplyAttributesToRange(rep.selStart, rep.selEnd,
						  [["openingDesignIsImage", "myId"],
						   ["openingDesignImageObject:foo1", "xyzzy"],
						   ["openingDesignImageObject:bar2", "naja"],
						   ]);
  }, "openingDesign", true);
}

/* used on the client side only */
openingDesign = new openingDesignInit();
