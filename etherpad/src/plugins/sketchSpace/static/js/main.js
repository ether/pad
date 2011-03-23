function sketchSpaceInit() {
  this.hooks = ['aceInitInnerdocbodyHead', 'aceAttribsToClasses', 'aceCreateDomLine'];
  this.padDocument = undefined;
}

sketchSpaceInit.prototype.aceInitInnerdocbodyHead = function(args) {
  args.iframeHTML.push('\'<link rel="stylesheet" type="text/css" href="/static/css/plugins/sketchSpace/ace.css"/>\'');
  args.iframeHTML.push('\'\\x3cscript type="text/javascript" src="/static/js/jquery-1.3.2.js">\\x3c/script>\'');
  args.iframeHTML.push('\'\\x3cscript type="text/javascript" src="/static/js/plugins/sketchSpace/ace_inner.js">\\x3c/script>\'');
};

sketchSpaceInit.prototype.aceAttribsToClasses = function(args) {
  if (args.key == 'sketchSpaceIsImage' && args.value != "")
    return ["sketchSpaceIsImage", "sketchSpaceImageId:" + args.value];
  else if (args.key.indexOf('sketchSpaceImageObject') == 0)
    return [args.key + ":" + args.value];
}

sketchSpaceInit.prototype.aceCreateDomLine = function(args) {
  if (args.cls.indexOf('sketchSpaceIsImage') >= 0) {
   var clss = [];
   var imageObjects = {};
   var imageId = undefined;
   $.each(args.cls.split(" "), function (i, cls) {
     if (cls.indexOf(":") != -1) {
       var key = cls.substr(0, cls.indexOf(":"));
       var val = cls.substr(cls.indexOf(":")+1);

       if (key == "sketchSpaceImageId") {
         clss.push("sketchSpaceImageId_" + val);
	 imageId = val;
       } else if (key == "sketchSpaceImageObject") {
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

   this.editorArea.images[imageId] = imageObjects;
   if (this.editorArea.currentImage == imageId) {
     this.updateImageFromPad();
   }

   return [{cls: clss.join(" "), extraOpenTags: '<a class="sketchSpaceImageLink">', extraCloseTags: '</a>'}];
  }
}

sketchSpaceInit.prototype.updateImageFromPad = function() {
  if (this.editorArea.currentImage != undefined) {
    var currentImage = this.editorArea.images[this.editorArea.currentImage];

    var visited = {};

    dojox.gfx.utils.forEach(sketchSpace.editorArea.surface, function (shape) {
      if (shape === sketchSpace.editorArea.surface || shape.objId === undefined) return;
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
	// FIXME: Handle that objStr is undefined here... can happen
	// if stuff changed between the loop above and this function.
        var obj = dojo.fromJson(objStr);

	var parent = sketchSpace.editorArea.surface;
	if (obj.parent) parent = materialize(obj.parent);

        var shape = dojox.gfx.utils.deserialize(parent, obj.shape);
	sketchSpace.editorArea.registerObjectShape(shape);

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

sketchSpaceInit.prototype.updatePadFromImage = function() {
  if (this.editorArea.currentImage != undefined) {
    var currentImage = this.editorArea.images[this.editorArea.currentImage];
    var imageLink = $(this.padDocument).find(".sketchSpaceImageId_" + this.editorArea.currentImage)[0];

    var visited = {};
    var update = [];

    dojox.gfx.utils.forEach(sketchSpace.editorArea.surface, function (shape) {
      if (shape === sketchSpace.editorArea.surface || shape.objId === undefined) return;
      if (currentImage[shape.objId] === undefined || currentImage[shape.objId] != shape.strRepr) {
        update.push(["sketchSpaceImageObject:" + shape.objId, escape(shape.strRepr)]);
      }
      visited[shape.objId] = shape;
    });

    for (var objId in currentImage)
      if (visited[objId] === undefined)
        update.push(["sketchSpaceImageObject:" + objId, ""]);

    padeditor.ace.callWithAce(function (ace) {
      ace.ace_performDocumentApplyAttributesToRange(ace.ace_getLineAndCharForPoint({node: imageLink, index:0, maxIndex:1}),
						    ace.ace_getLineAndCharForPoint({node: imageLink, index:1, maxIndex:1}),
						    update);
    }, "updatePadFromImage", true);

  }
}

sketchSpaceInit.prototype.selectImage = function(imageLink) {
  var imageId;
  $.each(imageLink.classList, function (idx, cls) {
    var parts = cls.split("_");
    if (parts[0] == "sketchSpaceImageId")
      imageId = parts[1];
  });

  this.padDocument = imageLink.ownerDocument;
  this.editorArea.currentImage = imageId;
  this.updateImageFromPad();
}

sketchSpaceInit.prototype.insertImage = function(event) {
  padeditor.ace.callWithAce(function (ace) {
    rep = ace.ace_getRep();

    ace.ace_replaceRange(rep.selStart, rep.selEnd, "I");
    ace.ace_performSelectionChange([rep.selStart[0],rep.selStart[1]-1], rep.selStart, false);
    ace.ace_performDocumentApplyAttributesToRange(rep.selStart, rep.selEnd,
						  [["sketchSpaceIsImage", dojox.uuid.generateRandomUuid()],
						   ["sketchSpaceImageObject:" + dojox.uuid.generateRandomUuid(),
						    escape('{parent:null,shape:{"shape":{"type":"circle","cx":100,"cy":100,"r":50},"stroke":{"type":"stroke","color":{"r":0,"g":255,"b":0,"a":1},"style":"solid","width":2,"cap":"butt","join":4},"fill":{"r":255,"g":0,"b":0,"a":1}}}')]
						   ]);
  }, "sketchSpace", true);
}

/* used on the client side only */
sketchSpace = new sketchSpaceInit();
