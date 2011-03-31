dojo.provide("sketchSpaceDesigner.designer.modes.Select");

dojo.require("sketchSpaceDesigner.designer.modes.Zoom");

dojo.declare("sketchSpaceDesigner.designer.modes.Select", [sketchSpaceDesigner.designer.modes.Zoom], {
  isOutlineMouseDown: false,
  isMoving: false,
  enable: function () {
    this.inherited(arguments);
    this.enableOutline();
    this.selectionUpdatedHandle = dojo.connect(this.designer.selection, "selectionUpdated", this, this.updateOutline);
    this.zoomHandle = dojo.connect(this.designer.surface_transform, "setTransform", this, this.updateOutline);
  },

  disable: function () {
    dojo.disconnect(this.zoomHandle);
    dojo.disconnect(this.selectionUpdatedHandle);
    this.disableOutline();
    this.inherited(arguments);
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

      this.outline.onMouseDownHandle = this.outline.connect("onmousedown", this, this.onOutlineMouseDown);
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

  onShapeMouseDown: function (shape, event) {
    this.inherited(arguments);
    this.onOutlineMouseDown(event);
  },

  onShapeMouseUp: function (shape, event) {
    this.inherited(arguments);
    if (!this.isOutlineMoving)
      this.designer.selection.toggleShape(shape, !event.ctrlKey);
  },

  onKeyUp: function (event) {
    this.inherited(arguments);
    if (event.keyCode == 46)
      this.designer.selection.applyToShapes("removeShape");
  },

  onMouseUp: function(event) {
    this.inherited(arguments);
    if (this.isOutlineMoving) {
      this.designer.selection.applyToShapes("save", this.getCurrentMove(event));
      this.isOutlineMoving = false;
    }
    this.isOutlineMouseDown = false;
  },

  onMouseMove: function(event) {
    this.inherited(arguments);

    if (!this.isOutlineMouseDown || !this.outline) return;

    this.isOutlineMoving = true;

    var move = this.getCurrentMove(event);
    this.outline.setTransform(dojox.gfx.matrix.multiply(this.outline.originalMatrix, move));

    move = this.getCurrentMove(event, this.designer.selection.parent, this.designer.selection.orig);
    this.designer.selection.applyToShapes(function () {
      this.setTransform(dojox.gfx.matrix.multiply(this.originalMatrix, move));
    });
  },

  getContainerShape: function () { return this.designer.surface; },

  onOutlineMouseDown: function(event) {
    this.isOutlineMouseDown = true;
    this.orig = this.getCurrentMouse(event);
    this.designer.selection.orig = this.getCurrentMouse(event, this.designer.selection.parent);
    if (!this.outline) return;
    this.outline.originalMatrix = this.outline.matrix;
    this.designer.selection.applyToShapes(function () {
      this.originalMatrix = this.matrix;
    });
  },

});
