dojo.provide("sketchSpaceDesigner.designer.modes.Mode");

dojo.require("dojox.gfx.matrix");

dojo.declare("sketchSpaceDesigner.designer.modes.Mode", [], {
  enable: function () {
    var mode = this;
    this.designer.forEachObjectShape(function (shape) { mode.enableShape(shape); });
  },
  disable: function () {
    var mode = this;
    this.designer.forEachObjectShape(function (shape) { mode.disableShape(shape); });
  },
  enableShape: function (shape) {
  },
  disableShape: function (shape) {
  },

  screenToLocalCoord: function (p) {
    var screenToObjMatrix = dojox.gfx.matrix.invert(this.getContainerShape()._getRealMatrix());
    return dojox.gfx.matrix.multiplyPoint(screenToObjMatrix, p.x, p.y);
  },

  getCurrentMouse: function (event) {
    return this.screenToLocalCoord({x:event.layerX, y:event.layerY});
  },

  getCurrentMove: function (event) {
    var mouse = this.getCurrentMouse(event);
    return dojox.gfx.matrix.translate(mouse.x - this.orig.x, mouse.y - this.orig.y);
  },

});
