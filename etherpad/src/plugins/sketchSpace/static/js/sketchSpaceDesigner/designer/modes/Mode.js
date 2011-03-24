dojo.provide("sketchSpaceDesigner.designer.modes.Mode");

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
  }
});
