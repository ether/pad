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
	 var objId = val.substr(0, val.indexOf(":"));
	 var properties = val.substr(val.indexOf(":")+1);	 
	 imageObjects[objId] = unescape(properties);
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

    var visited = {};

    dojox.gfx.utils.forEach(openingDesign.editorArea.surface, function (shape) {
      if (shape === openingDesign.editorArea.surface) return;
      if (currentImage[shape.objId] === undefined) {
       shape.removeShape();
      } else {
        if (shape.strRepr == currentImage[shape.objId]) {
          visited[shape.objId] = shape;
        } else {
	  shape.removeShape();
        }
      }
    });

    function materialize (objId) {
      if (visited[objId] === undefined) {
        var objStr = currentImage[objId];
        var obj = dojo.fromJson(objStr);

	var parent = openingDesign.editorArea.surface;
	if (obj.parent) parent = materialize(obj.parent);

        var shape = dojox.gfx.utils.deserialize(parent, obj.shape);
        shape.objId = objId;
        shape.strRepr = objStr;
	visited[objId] = shape;
      }
      return visited[objId];
    }

    for (var objId in currentImage)
      materialize(objId);
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
						   ["openingDesignImageObject:foo",
						    escape('{parent:null,shape:{"shape":{"type":"circle","cx":100,"cy":100,"r":50},"stroke":{"type":"stroke","color":{"r":255,"g":0,"b":0,"a":1},"style":"solid","width":2,"cap":"butt","join":4},"fill":{"r":255,"g":0,"b":0,"a":1}}}')]
						   ]);
  }, "openingDesign", true);
}

/* used on the client side only */
openingDesign = new openingDesignInit();
