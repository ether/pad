dojo.provide("sketchSpaceDesigner.designer.selection");

dojo.require("sketchSpaceDesigner.designer.bbox");

dojo.declare("sketchSpaceDesigner.designer.selection.Selection", [], {
  constructor: function (designer) {
    this.designer = designer;
    this.objects = {};
    this.parent = undefined;
    this.outline = undefined;
  },

  editorSelectionBbox: function() {
    var bbox = new sketchSpaceDesigner.designer.bbox.Bbox();
    for (objId in this.objects) {
      bbox.addPoints(this.objects[objId].getTransformedBoundingBox());
    }
    return bbox;
  },

  editorSelectionUpdateOutline: function() {
    var selection = this;

    var bbox = this.editorSelectionBbox();

    if (this.outline !== undefined) {
      this.outline.removeShape();
      this.outline = undefined;
    }

    if (bbox.x !== undefined) {
      this.outline = this.designer.surface.createGroup();

      this.outline.setTransform(dojox.gfx.matrix.translate(bbox.x, bbox.y));
      this.outline.originalMatrix = this.outline.matrix;

      this.outline.outlineRect = dojox.gfx.utils.deserialize(this.outline, {shape:{type:"rect", x:0, y:0, width:bbox.width, height:bbox.height}, stroke:{color:{r:196,g:196,b:196,a:1},width:1, style:"solid"}});

      this.outline.outlineCornerTL = dojox.gfx.utils.deserialize(this.outline, {shape:{type:"rect", x:-2, y:-2, width:4, height:4}, stroke:{color:{r:128,g:128,b:128,a:1},width:1}, fill:{r:196,g:196,b:196,a:1}});
      this.outline.outlineCornerBL = dojox.gfx.utils.deserialize(this.outline, {shape:{type:"rect", x:-2, y:bbox.height-2, width:4, height:4}, stroke:{color:{r:128,g:128,b:128,a:1},width:1}, fill:{r:196,g:196,b:196,a:1}});
      this.outline.outlineCornerTH = dojox.gfx.utils.deserialize(this.outline, {shape:{type:"rect", x:bbox.width-2, y:-2, width:4, height:4}, stroke:{color:{r:128,g:128,b:128,a:1},width:1}, fill:{r:196,g:196,b:196,a:1}});
      this.outline.outlineCornerBH = dojox.gfx.utils.deserialize(this.outline, {shape:{type:"rect", x:bbox.width-2, y:bbox.height-2, width:4, height:4}, stroke:{color:{r:128,g:128,b:128,a:1},width:1}, fill:{r:196,g:196,b:196,a:1}});

      this.moveable = new dojox.gfx.Moveable(this.outline);
      this.movedSignalHandle = dojo.connect(this.moveable, "onMoveStop", this, this.movedCallback);
      this.clickSignalHandle = this.outline.connect("onclick", this.outline, function (event) { selection.clickCallback(this, event); });
    }
  },

  movedCallback: function(mover) {
   var matrix = dojox.gfx.matrix.multiply(this.outline.matrix, dojox.gfx.matrix.invert(this.outline.originalMatrix));
    this.outline.originalMatrix = this.outline.matrix;

    for (objId in this.objects) {
      this.objects[objId].applyLeftTransform(matrix);
      this.designer.saveShapeToStr(this.objects[objId]);
    }
    this.designer.imageUpdated();
  },

  clickCallback: function(shape, event) {
  },

  editorShapeAddToSelection: function(shape) {
    if (shape.objId === undefined) return;
    if (this.parent !== shape.parent) {
      this.objects = {};
      this.parent = shape.parent;
    }
    this.objects[shape.objId] = shape;
    this.editorSelectionUpdateOutline();
  },

  editorShapeIsSelected: function(shape) {
    return shape.objId !== undefined && this.objects[shape.objId] !== undefined;
  },

  editorShapeRemoveFromSelection: function(shape) {
    if (shape.objId === undefined || this.objects[shape.objId] === undefined) return;
    delete this.objects[shape.objId];
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
    this.objects = {};
    this.editorSelectionUpdateOutline();
  },


  /* Operations on the selected objects */

  editorSelectionShapeRemove: function() {
    for (objId in this.objects) {
      this.designer.editorShapeRemove(this.objects[objId]);
    }
    this.editorShapeClearSelection();
  }
});
