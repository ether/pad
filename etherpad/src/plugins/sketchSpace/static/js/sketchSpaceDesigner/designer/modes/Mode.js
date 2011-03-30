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

  screenToLocalCoord: function (p, container) {
    if (container === undefined) container = this.getContainerShape();
    var screenToObjMatrix = dojox.gfx.matrix.invert(container._getRealMatrix());
    return dojox.gfx.matrix.multiplyPoint(screenToObjMatrix, p.x, p.y);
  },

   getCurrentMouse: function (event, container) {
   return this.screenToLocalCoord({x:event.layerX, y:event.layerY}, container);
  },

  getCurrentMove: function (event, container, orig) {
    var mouse = this.getCurrentMouse(event, container);
    if (orig === undefined) orig = this.orig;
    return dojox.gfx.matrix.translate(mouse.x - orig.x, mouse.y - orig.y);
  },

});
