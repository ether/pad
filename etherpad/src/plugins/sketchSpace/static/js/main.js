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
   var argClss = args.cls.split(" ");

   for (var i = 0; i < argClss.length; i++) {
     var cls = argClss[i];
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
   }

   this.editorArea.images[imageId] = imageObjects;
   if (this.editorArea.currentImage == imageId) {
     this.updateImageFromPad();
   }

   return [{cls: clss.join(" "), extraOpenTags: '<a class="sketchSpaceImageLink">', extraCloseTags: '</a>'}];
  }
}

sketchSpaceInit.prototype.updateImageFromPad = function() {
  if (this.editorArea.currentImage !== undefined) {
    var currentImage = this.editorArea.images[this.editorArea.currentImage];

    /* Some debug info printing:
    console.log("Image:");
    sketchSpace.editorArea.forEachObjectShape(function (shape) { console.log(shape.objId); })
    console.log("Pad:");
    for (name in currentImage)
      console.log(name);
    */

    var visited = {};
    var toDelete = {};

    this.editorArea.forEachObjectShape(function (shape) {
      if (currentImage[shape.objId] === undefined) {
        toDelete[shape.objId] = shape;
      } else {
        if (shape.strRepr == currentImage[shape.objId]) {
          visited[shape.objId] = shape;
        } else {
          toDelete[shape.objId];
        }
      }
    });

    for (objId in toDelete) {
      toDelete[objId].removeShape();
    }

    function materialize (objId) {
      if (visited[objId] === undefined) {
        var objStr = currentImage[objId];
	// FIXME: Handle that objStr is undefined here... can happen
	// if stuff changed between the loop above and this function.
        var obj = dojo.fromJson(objStr);

	var parent = sketchSpace.editorArea.surface_transform;
	if (obj.parent) parent = materialize(obj.parent);

        var shape = sketchSpace.editorArea.deserializeShape(parent, obj.shape);
	sketchSpace.editorArea.registerObjectShape(shape);

        shape.objId = objId;
        shape.strRepr = objStr;
	visited[objId] = shape;
      }
      return visited[objId];
    }

    for (var objId in currentImage)
      materialize(objId);

    this.editorArea.imageUpdatedByOthers();
  }
}

sketchSpaceInit.prototype.updatePadFromImage = function() {
  if (this.editorArea.currentImage !== undefined) {
    var currentImageId = this.editorArea.currentImage;
    var currentImage = this.editorArea.images[currentImageId];

    var visited = {};
    var update = [];

    this.editorArea.forEachObjectShape(function (shape) {
      if (currentImage[shape.objId] === undefined || currentImage[shape.objId] != shape.strRepr) {
        update.push(["sketchSpaceImageObject:" + shape.objId, escape(shape.strRepr)]);
      }
      visited[shape.objId] = shape;
    });

    for (var objId in currentImage)
      if (visited[objId] === undefined)
        update.push(["sketchSpaceImageObject:" + objId, ""]);

    this.updatePad(currentImageId, update);
  }
}

sketchSpaceInit.prototype.updatePad = function (imageId, update) {
  var sketchSpace = this;
  padeditor.ace.callWithAce(function (ace) {
    sketchSpace.ace_updatePad(ace, imageId, update);
  }, "updatePadFromImage", true);
}

sketchSpaceInit.prototype.getImageLinkFromId = function (imageId) {
  return $(this.padDocument).find(".sketchSpaceImageId_" + imageId)[0];
}

sketchSpaceInit.prototype.selectImage = function(imageLink) {
  var imageId;
  $.each(imageLink.classList, function (idx, cls) {
    var parts = cls.split("_");
    if (parts[0] == "sketchSpaceImageId")
      imageId = parts[1];
  });

  this.padDocument = imageLink.ownerDocument;
  this.editorArea.selectImage(imageId);
  this.updateImageFromPad();
}

sketchSpaceInit.prototype.insertImage = function() {
  var sketchSpace = this;
  
  return padeditor.ace.callWithAce(function (ace) {
    return sketchSpace.ace_insertImage(ace);
  }, "sketchSpace", true);
}

sketchSpaceInit.prototype.ace_getImageRange = function (ace, imageId) {
  var imageLink = this.getImageLinkFromId(imageId);
  return [ace.ace_getLineAndCharForPoint({node: imageLink, index:0, maxIndex:1}),
	  ace.ace_getLineAndCharForPoint({node: imageLink, index:1, maxIndex:1})];
}

sketchSpaceInit.prototype.ace_updatePad = function (ace, imageId, update) {
  var imageRange = this.ace_getImageRange(ace, imageId);
  ace.ace_performDocumentApplyAttributesToRange(imageRange[0], imageRange[1], update);
}

sketchSpaceInit.prototype.ace_insertImage = function(ace) {
  var imageId = dojox.uuid.generateRandomUuid();
  rep = ace.ace_getRep();

  ace.ace_replaceRange(rep.selStart, rep.selEnd, "I");
  ace.ace_performSelectionChange([rep.selStart[0],rep.selStart[1]-1], rep.selStart, false);
  ace.ace_performDocumentApplyAttributesToRange(rep.selStart, rep.selEnd, [["sketchSpaceIsImage", imageId]]);

  return imageId;
}

/* used on the client side only */
sketchSpace = new sketchSpaceInit();
