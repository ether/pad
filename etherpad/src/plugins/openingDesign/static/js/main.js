function openingDesignInit() {
  this.hooks = ['aceInitInnerdocbodyHead', 'aceAttribsToClasses', 'aceCreateDomLine'];
  this.images = {};
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
     if (cls.indexOf(":") != -1) {
       var key = cls.substr(0, cls.indexOf(":"));
       var val = cls.substr(cls.indexOf(":")+1);

       if (key == "openingDesignImageId") {
         clss.push(cls);
	 imageId = val;
       } else if (key == "openingDesignImageObject") {
	 var id = val.substr(0, val.indexOf(":"));
	 var properties = val.substr(val.indexOf(":")+1);	 
	 imageObjects[id] = dojo.fromJson(unescape(properties));
       } else {
         clss.push(cls);
       }

     } else {
       clss.push(cls);
     }
   });

   this.images[imageId] = imageObjects;
   if (this.currentImage.id == imageId) {
     this.updateImageFromPad();
   }

   return [{cls: clss.join(" "), extraOpenTags: '<a class="openingDesignImageLink">', extraCloseTags: '</a>'}];
  }
}

openingDesignInit.prototype.updateImageFromPad = function() {
  if (this.currentImage.id != undefined) {
    var currentImage = this.images[this.currentImage.id];

    console.log({updateImageFromPad:currentImage});

/*

shape = openingDesign.editorArea.surface.createCircle({cx: cx, cy: cy, r: r})
			   .setFill(randColor(true))
			   .setStroke({color: randColor(true), width: getRand(0, 3)})

*/
  }
}

openingDesignInit.prototype.selectImage = function(imageLink) {
  var imageId;
  $.each(imageLink.classList, function (idx, cls) {
    var parts = cls.split(":");
    if (parts[0] == "openingDesignImageId")
      imageId = parts[1];
  });

  /*
  if (this.currentImage.link != undefined) {
    $(this.currentImage.link).removeClass("selected");
  }
  $(imageLink).addClass("selected");
  */
  this.currentImage = {'id': imageId, 'link': imageLink};
  this.updateImageFromPad();
}

openingDesignInit.prototype.insertImage = function(event) {
  padeditor.ace.callWithAce(function (ace) {
    rep = ace.ace_getRep();

    ace.ace_replaceRange(rep.selStart, rep.selEnd, "I");
    ace.ace_performSelectionChange([rep.selStart[0],rep.selStart[1]-1], rep.selStart, false);
    ace.ace_performDocumentApplyAttributesToRange(rep.selStart, rep.selEnd,
						  [["openingDesignIsImage", "myId"],
						   ["openingDesignImageObject:foo", escape(dojo.toJson({type:'circle', x:100, y:100, r:50, fill:[255, 0, 0, 1.0]}))]						  
						   ]);
  }, "openingDesign", true);
}

/* used on the client side only */
openingDesign = new openingDesignInit();
