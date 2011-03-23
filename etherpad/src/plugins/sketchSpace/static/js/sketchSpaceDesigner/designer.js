dojo.provide("sketchSpaceDesigner.designer");

dojo.require("sketchSpaceDesigner.designer.bbox");
dojo.require("sketchSpaceDesigner.designer.selection");
dojo.require("dojox.gfx");
dojo.require("dojox.gfx.move");
dojo.require("dojox.gfx.utils");
dojo.require("dojox.gfx.matrix");
dojo.require("dojox.uuid.generateRandomUuid");

dojo.declare("sketchSpaceDesigner.designer.Designer", [], {
 constructor: function (container, width, height) {
    this.container = container;
    this.surface = dojox.gfx.createSurface(this.container, width, height);
    this.surface_size = {width: width, height: height};

    this.images = {};
    this.currentImage = undefined;
    this.selection = new sketchSpaceDesigner.designer.selection.Selection();

    dojo.connect(this.container, "ondragstart",   dojo, "stopEvent");
    dojo.connect(this.container, "onselectstart", dojo, "stopEvent");
  },

  saveShapeToStr: function(shape) {
    var parent = null;
    if (shape.parent.objId != undefined)
      parent = shape.parent.objId;

    shape.strRepr = dojo.toJson({parent:parent, shape:dojox.gfx.utils.serialize(shape)});
  },

  /* Use this to listen for changes */
  imageUpdated: function () {},

  editorGetShapeByObjId: function(objId) {
    var designer = this;
    if (objId == null) return this.surface;
    var res = undefined;
    dojox.gfx.utils.forEach(this.surface, function (shape) {
      if (shape === designer.surface) return;
      if (shape.objId == objId) res = shape;
    });
    return res;
  },

  editorShapeMakeMoveable: function(shape) {
    var designer = this;
    shape.moveable = new dojox.gfx.Moveable(shape);
    shape.shapeMovedSignalHandle = dojo.connect(shape.moveable, "onMoveStop", this, this.editorCallbackShapeMoved);
    shape.clickSignalHandle = shape.connect("onclick", shape, function (event) { designer.editorCallbackShapeClick(this, event); });
  },

  editorCallbackShapeMoved: function(mover) {
    this.saveShapeToStr(mover.host.shape);
    this.imageUpdated();
  },

  editorCallbackShapeClick: function(shape, event) {
    this.editorShapeToggleSelection(shape, !event.ctrlKey);
  },

  editorShapeRemove: function(shape) {
    shape.removeShape();
    this.imageUpdated();
  },

  editorAddShape: function(shapeDescription) {
    var shape = dojox.gfx.utils.deserialize(this.editorGetShapeByObjId(shapeDescription.parent), shapeDescription.shape);
    shape.objId = dojox.uuid.generateRandomUuid();
    this.editorShapeMakeMoveable(shape);
    this.saveShapeToStr(shape);
    this.imageUpdated();
  },

  editorAddCircle: function() {
    this.editorAddShape({parent:null,shape:{"shape":{"type":"circle","cx":100,"cy":100,"r":50},"stroke":{"type":"stroke","color":{"r":0,"g":255,"b":0,"a":1},"style":"solid","width":2,"cap":"butt","join":4},"fill":{"r":255,"g":0,"b":0,"a":1}}});
  },

  editorSelectionBbox: function() {
    var bbox = new sketchSpaceDesigner.designer.bbox.Bbox();
    for (objId in this.selection.objects) {
      bbox.addPoints(this.selection.objects[objId].getTransformedBoundingBox());
    }
    return bbox;
  },

  editorSelectionUpdateOutline: function() {
    var bbox = this.editorSelectionBbox();

    if (this.selection.outline !== undefined) {
      this.selection.outline.removeShape();
      this.selection.outline = undefined;
    }

    if (bbox.x !== undefined) {
      this.selection.outline = this.surface.createGroup();

      this.selection.outline.setTransform(dojox.gfx.matrix.translate(bbox.x, bbox.y));

      this.selection.outline.outlineRect = dojox.gfx.utils.deserialize(this.selection.outline, {shape:{type:"rect", x:0, y:0, width:bbox.width, height:bbox.height}, stroke:{color:{r:196,g:196,b:196,a:1},width:1, style:"solid"}});

      this.selection.outline.outlineCornerTL = dojox.gfx.utils.deserialize(this.selection.outline, {shape:{type:"rect", x:-2, y:-2, width:4, height:4}, stroke:{color:{r:128,g:128,b:128,a:1},width:1}, fill:{r:196,g:196,b:196,a:1}});
      this.selection.outline.outlineCornerBL = dojox.gfx.utils.deserialize(this.selection.outline, {shape:{type:"rect", x:-2, y:bbox.height-2, width:4, height:4}, stroke:{color:{r:128,g:128,b:128,a:1},width:1}, fill:{r:196,g:196,b:196,a:1}});
      this.selection.outline.outlineCornerTH = dojox.gfx.utils.deserialize(this.selection.outline, {shape:{type:"rect", x:bbox.width-2, y:-2, width:4, height:4}, stroke:{color:{r:128,g:128,b:128,a:1},width:1}, fill:{r:196,g:196,b:196,a:1}});
      this.selection.outline.outlineCornerBH = dojox.gfx.utils.deserialize(this.selection.outline, {shape:{type:"rect", x:bbox.width-2, y:bbox.height-2, width:4, height:4}, stroke:{color:{r:128,g:128,b:128,a:1},width:1}, fill:{r:196,g:196,b:196,a:1}});
    }
  },

  editorShapeAddToSelection: function(shape) {
    if (shape.objId === undefined) return;
    if (this.selection.parent !== shape.parent) {
      this.selection.objects = {};
      this.selection.parent = shape.parent;
    }
    this.selection.objects[shape.objId] = shape;
    this.editorSelectionUpdateOutline();
  },

  editorShapeIsSelected: function(shape) {
    return shape.objId !== undefined && this.selection.objects[shape.objId] !== undefined;
  },

  editorShapeRemoveFromSelection: function(shape) {
    if (shape.objId === undefined || this.selection.objects[shape.objId] === undefined) return;
    delete this.selection.objects[shape.objId];
    this.editorSelectionUpdateOutline();
  },

  editorShapeToggleSelection: function(shape, clearOthers) {
    var isSelected = this.editorShapeIsSelected(shape);
    if (clearOthers)
      this.editorShapeClearSelection();
    if (isSelected)
      this.editorShapeRemoveFromSelection(shape);
    else
      this.editorShapeAddToSelection(shape);
  },

  editorShapeClearSelection: function () {
    this.selection.objects = {};
    this.editorSelectionUpdateOutline();
  },

  editorSelectionShapeRemove: function() {
    for (objId in this.selection.objects) {
      this.editorShapeRemove(this.selection.objects[objId]);
    }
    this.editorShapeClearSelection();
  }


});

dojo.addOnLoad(function (){
  sketchSpace.editorArea = new sketchSpaceDesigner.designer.Designer(dojo.byId("sketchSpaceDebug"), 300, 300);
  dojo.connect(sketchSpace.editorArea, "imageUpdated", sketchSpace, sketchSpace.updatePadFromImage);
});
