dojo.provide("sketchSpaceDesigner.designer.modes.Select");

dojo.require("sketchSpaceDesigner.designer.modes.Zoom");

dojo.declare("sketchSpaceDesigner.designer.modes.Select", [sketchSpaceDesigner.designer.modes.Zoom], {
  enable: function () {
    var mode = this;
    this.inherited(arguments);
    this.onKeyUpHandle = dojo.connect(document, "onkeyup", this, function (event) { mode.onKeyUp(event); });
    this.enableOutline();
    this.selectionUpdatedHandle = dojo.connect(this.designer.selection, "selectionUpdated", this, this.updateOutline);
    this.zoomHandle = dojo.connect(this.designer.surface_transform, "setTransform", this, this.updateOutline);
  },

  disable: function () {
    dojo.disconnect(this.zoomHandle);
    dojo.disconnect(this.selectionUpdatedHandle);
    this.disableOutline();
    dojo.disconnect(this.onKeyUpHandle);
    this.inherited(arguments);
  },

  enableShape: function (shape) {
    var mode = this;
    shape.onShapeClickHandle = shape.connect("onclick", shape, function (event) { mode.onShapeClick(shape, event); });
  },

  disableShape: function (shape) {
    dojo.disconnect(shape.onClickHandle);
  },

  onShapeClick: function (shape, event) {
    this.designer.selection.toggleShape(shape, !event.ctrlKey);
  },

  onKeyUp: function (event) {
    if (event.keyCode == 46)
     this.designer.selection.applyToShapes("removeShape");
  },

  enableOutline: function() {
    var bbox = this.designer.selection.getBbox();

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
      this.isMoving = false;
      this.onMoveStartSignalHandle = dojo.connect(this.moveable, "onFirstMove", this, this.onFirstMove);
      this.onMoveStopSignalHandle = dojo.connect(this.moveable, "onMoveStop", this, this.onMoveStop);
      this.enableClick();
    }
  },

  disableOutline: function () {
    if (this.outline !== undefined) {
      this.outline.removeShape();
      this.outline = undefined;
    }
  },

  updateOutline: function () {
    this.disableOutline();
    this.enableOutline();
  },

  enableClick: function () {
    var selection = this;
    this.clickSignalHandle = this.outline.connect("onclick", this.outline, function (event) { selection.onClick(this, event); });
  },

  disableClick: function () {
    dojo.disconnect(this.clickSignalHandle);
  },

  onFirstMove: function() {
    this.disableClick();
    this.isMoving = true;
  },

  onMoveStop: function(mover) {
    if (!this.isMoving) return;
    this.isMoving = false;
    this.onMove(mover);
    var selection = this;
    setTimeout(function () { selection.enableClick(); }, 1);
  },

  onMove: function(mover) {
    var matrix = dojox.gfx.matrix.multiply(this.outline.matrix, dojox.gfx.matrix.invert(this.outline.originalMatrix));
    this.outline.originalMatrix = this.outline.matrix;
    this.designer.selection.applyToShapes("applyLeftTransform", matrix);
  },

  onClick: function(shape, event) {
    console.log("CLICK");
  },

});
