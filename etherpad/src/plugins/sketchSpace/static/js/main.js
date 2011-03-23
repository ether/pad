function sketchSpaceInit() {
  this.hooks = ['aceInitInnerdocbodyHead', 'aceAttribsToClasses', 'aceCreateDomLine'];
  this.images = {};
  this.padDocument = undefined;
  this.currentImage = undefined;
  this.selection = {objects:{}, parent: undefined, outline:undefined};
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

   this.images[imageId] = imageObjects;
   if (this.currentImage == imageId) {
     this.updateImageFromPad();
   }

   return [{cls: clss.join(" "), extraOpenTags: '<a class="sketchSpaceImageLink">', extraCloseTags: '</a>'}];
  }
}

sketchSpaceInit.prototype.updateImageFromPad = function() {
  if (this.currentImage != undefined) {
    var currentImage = this.images[this.currentImage];

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
	sketchSpace.editorShapeMakeMoveable(shape);

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

sketchSpaceInit.prototype.saveShapeToStr = function(shape) {
  var parent = null;
  if (shape.parent.objId != undefined)
    parent = shape.parent.objId;

  shape.strRepr = dojo.toJson({parent:parent, shape:dojox.gfx.utils.serialize(shape)});
}

sketchSpaceInit.prototype.updatePadFromImage = function() {
  if (this.currentImage != undefined) {
    var currentImage = this.images[this.currentImage];
    var imageLink = $(this.padDocument).find(".sketchSpaceImageId_" + this.currentImage)[0];

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
        update.push(["sketchSpaceImageObject:" + shape.objId, ""]);

    padeditor.ace.callWithAce(function (ace) {
      ace.ace_performDocumentApplyAttributesToRange(ace.ace_getLineAndCharForPoint({node: imageLink, index:0, maxIndex:1}),
						    ace.ace_getLineAndCharForPoint({node: imageLink, index:1, maxIndex:1}),
						    update);
    }, "updatePadFromImage", true);

  }
}

sketchSpaceInit.prototype.editorGetShapeByObjId = function(objId) {
  if (objId == null) return sketchSpace.editorArea.surface;
  var res = undefined;
  dojox.gfx.utils.forEach(sketchSpace.editorArea.surface, function (shape) {
    if (shape === sketchSpace.editorArea.surface) return;
    if (shape.objId == objId) res = shape;
  });
  return res;
}

sketchSpaceInit.prototype.editorShapeMakeMoveable = function(shape) {
  shape.moveable = new dojox.gfx.Moveable(shape);
  shape.shapeMovedSignalHandle = dojo.connect(shape.moveable, "onMoveStop", this, this.editorCallbackShapeMoved);
  shape.clickSignalHandle = shape.connect("onclick", shape, function (event) { sketchSpace.editorCallbackShapeClick(this, event); });
}

sketchSpaceInit.prototype.editorCallbackShapeMoved = function(mover) {
  this.saveShapeToStr(mover.host.shape);
  this.updatePadFromImage();
}

sketchSpaceInit.prototype.editorCallbackShapeClick = function(shape, event) {
  if (event.ctrlKey)
    this.editorShapeToggleSelection(shape);
}

sketchSpaceInit.prototype.editorAddShape = function(shapeDescription) {
  var shape = dojox.gfx.utils.deserialize(this.editorGetShapeByObjId(shapeDescription.parent), shapeDescription.shape);
  shape.objId = dojox.uuid.generateRandomUuid();
  this.editorShapeMakeMoveable(shape);
  this.saveShapeToStr(shape);
  this.updatePadFromImage();
}

sketchSpaceInit.prototype.editorAddCircle = function() {
  this.editorAddShape({parent:null,shape:{"shape":{"type":"circle","cx":100,"cy":100,"r":50},"stroke":{"type":"stroke","color":{"r":0,"g":255,"b":0,"a":1},"style":"solid","width":2,"cap":"butt","join":4},"fill":{"r":255,"g":0,"b":0,"a":1}}});
}

sketchSpaceInit.prototype.mergeBbox = function(bbox1, bbox2) {
  var res = {};
  res.x = min(bbox1.x, bbox2.x);
  res.y = min(bbox1.y, bbox2.y);

  res.width = max(bbox1.x + bbox1.width, bbox2.x + bbox2.width) - res.x;
  res.height = max(bbox1.y + bbox1.height, bbox2.y + bbox2.height) - res.y;
  return res;
}

sketchSpaceInit.prototype.bboxAddPoints = function(bbox, points) {
  var res = undefined;
  if (bbox !== undefined) {
    res = {x:bbox.x, y:bbox.y, width:bbox.width, height:bbox.height};
  }
  $.each(points, function (index, point) {
    if (res === undefined) {
      res = {x:point.x, y:point.y, width:0, height:0};
    } else {
      if (point.x < res.x) {
        res.width += res.x - point.x;
        res.x = point.x;
      } else if (point.x > res.x + res.width) {
        res.width = point.x - res.x;
      }
      if (point.y < res.y) {
        res.height += res.y - point.y;
        res.y = point.y;
      } else if (point.y > res.y + res.height) {
        res.height = point.y - res.y;
      }
    }
  });
  return res;
}

sketchSpaceInit.prototype.editorSelectionBbox = function() {
  var bbox = undefined;
  for (objId in this.selection.objects) {
    bbox = this.bboxAddPoints(bbox, this.selection.objects[objId].getTransformedBoundingBox());
  }
  return bbox;
}

sketchSpaceInit.prototype.editorSelectionUpdateOutline = function() {
  var bbox = this.editorSelectionBbox();

  if (this.selection.outline !== undefined) {
    this.selection.outline.removeShape();
    this.selection.outline = undefined;
  }

  if (bbox !== undefined) {
    this.selection.outline = this.editorArea.surface.createGroup();

    this.selection.outline.setTransform(dojox.gfx.matrix.translate(bbox.x, bbox.y));

    this.selection.outline.outlineRect = dojox.gfx.utils.deserialize(this.selection.outline, {shape:{type:"rect", x:0, y:0, width:bbox.width, height:bbox.height}, stroke:{color:{r:196,g:196,b:196,a:1},width:1, style:"solid"}});

    this.selection.outline.outlineCornerTL = dojox.gfx.utils.deserialize(this.selection.outline, {shape:{type:"rect", x:-2, y:-2, width:4, height:4}, stroke:{color:{r:128,g:128,b:128,a:1},width:1}, fill:{r:196,g:196,b:196,a:1}});
    this.selection.outline.outlineCornerBL = dojox.gfx.utils.deserialize(this.selection.outline, {shape:{type:"rect", x:-2, y:bbox.height-2, width:4, height:4}, stroke:{color:{r:128,g:128,b:128,a:1},width:1}, fill:{r:196,g:196,b:196,a:1}});
    this.selection.outline.outlineCornerTH = dojox.gfx.utils.deserialize(this.selection.outline, {shape:{type:"rect", x:bbox.width-2, y:-2, width:4, height:4}, stroke:{color:{r:128,g:128,b:128,a:1},width:1}, fill:{r:196,g:196,b:196,a:1}});
    this.selection.outline.outlineCornerBH = dojox.gfx.utils.deserialize(this.selection.outline, {shape:{type:"rect", x:bbox.width-2, y:bbox.height-2, width:4, height:4}, stroke:{color:{r:128,g:128,b:128,a:1},width:1}, fill:{r:196,g:196,b:196,a:1}});
  }
}

sketchSpaceInit.prototype.editorShapeAddToSelection = function(shape) {
  if (shape.objId === undefined) return;
  if (this.selection.parent !== shape.parent) {
    this.selection.objects = {};
    this.selection.parent = shape.parent;
  }
  this.selection.objects[shape.objId] = shape;
  this.editorSelectionUpdateOutline();
}

sketchSpaceInit.prototype.editorShapeRemoveFromSelection = function(shape) {
  if (shape.objId === undefined || this.selection.objects[shape.objId] === undefined) return;
  delete this.selection.objects[shape.objId];
  this.editorSelectionUpdateOutline();
}

sketchSpaceInit.prototype.editorShapeToggleSelection = function(shape) {
  if (shape.objId === undefined) return;
  if (this.selection.objects[shape.objId] === undefined)
    this.editorShapeAddToSelection(shape);
  else
    this.editorShapeRemoveFromSelection(shape);
}

sketchSpaceInit.prototype.editorShapeClearSelection = function () {
  this.selection.objects = {};
  this.editorSelectionUpdateOutline();
}

sketchSpaceInit.prototype.selectImage = function(imageLink) {
  var imageId;
  $.each(imageLink.classList, function (idx, cls) {
    var parts = cls.split("_");
    if (parts[0] == "sketchSpaceImageId")
      imageId = parts[1];
  });

  this.padDocument = imageLink.ownerDocument;
  this.currentImage = imageId;
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
