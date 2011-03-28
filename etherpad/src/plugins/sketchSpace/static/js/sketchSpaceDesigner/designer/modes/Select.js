dojo.provide("sketchSpaceDesigner.designer.modes.Select");

dojo.require("sketchSpaceDesigner.designer.modes.Zoom");

dojo.declare("sketchSpaceDesigner.designer.modes.Select", [sketchSpaceDesigner.designer.modes.Zoom], {
  isMouseDown: false,
  isMoving: false,
  enable: function () {
    var mode = this;
    this.inherited(arguments);
    this.onKeyUpHandle = dojo.connect(document, "onkeyup", this, function (event) { mode.onKeyUp(event); });
    this.enableOutline();
    this.selectionUpdatedHandle = dojo.connect(this.designer.selection, "selectionUpdated", this, this.updateOutline);
    this.zoomHandle = dojo.connect(this.designer.surface_transform, "setTransform", this, this.updateOutline);
    this.onMouseMoveHandle = dojo.connect(this.designer.container, "onmousemove", this, this.onMouseMove);
    this.onMouseUpHandle = dojo.connect(this.designer.container, "onmouseup", this, this.onMouseUp);
  },

  disable: function () {
    dojo.disconnect(this.onMouseUpHandle);
    dojo.disconnect(this.onMouseMoveHandle);
    dojo.disconnect(this.zoomHandle);
    dojo.disconnect(this.selectionUpdatedHandle);
    this.disableOutline();
    dojo.disconnect(this.onKeyUpHandle);
    this.inherited(arguments);
  },

  enableShape: function (shape) {
    var mode = this;
    shape.onMouseUpHandle = shape.connect("onmouseup", shape, function (event) { mode.onShapeMouseUp(shape, event); });
    shape.onMouseDownHandle = shape.connect("onmousedown", shape, function (event) { mode.onShapeMouseDown(shape, event); });
  },

  disableShape: function (shape) {
    dojo.disconnect(shape.onMouseUpHandle);
    dojo.disconnect(shape.onMouseDownHandle);
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

      this.onMouseUpHandle = this.outline.connect("onmouseup", this, this.onMouseUp);
      this.onMouseDownHandle = this.outline.connect("onmousedown", this, this.onMouseDown);
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

  onShapeMouseUp: function (shape, event) {
    this.designer.selection.toggleShape(shape, !event.ctrlKey);
  },

  onShapeMouseDown: function (shape, event) {
  },

  onKeyUp: function (event) {
    if (event.keyCode == 46)
      this.designer.selection.applyToShapes("removeShape");
  },

  getCurrentMouse: function (event) {
    var screenToObjMatrix = dojox.gfx.matrix.invert(this.outline.parent._getRealMatrix());
    var mouse = dojox.gfx.matrix.multiplyPoint(screenToObjMatrix, event.layerX, event.layerY);
    return mouse;
  },

  getCurrentMove: function (event) {
    var mouse = this.getCurrentMouse(event);
    return dojox.gfx.matrix.translate(mouse.x - this.orig.x, mouse.y - this.orig.y);
  },

  onMouseUp: function(event) {
    if (!this.isMoving) {
      console.log("CLICK");
    } else {
      this.designer.selection.applyToShapes("save", this.getCurrentMove(event));
    }
    this.isMoving = false;
    this.isMouseDown = false;
  },

  onMouseDown: function(event) {
    this.orig = this.getCurrentMouse(event);
    this.outline.originalMatrix = this.outline.matrix;
    this.designer.selection.applyToShapes(function () {
      this.originalMatrix = this.matrix;
    });
    this.isMouseDown = true;
  },

  onMouseMove: function(event) {
    if (!this.isMouseDown) return;
    this.isMoving = true;
    var move = this.getCurrentMove(event);
    this.outline.setTransform(dojox.gfx.matrix.multiply(this.outline.originalMatrix, move));
    this.designer.selection.applyToShapes(function () {
      this.setTransform(dojox.gfx.matrix.multiply(this.originalMatrix, move));
    });
  },

});
