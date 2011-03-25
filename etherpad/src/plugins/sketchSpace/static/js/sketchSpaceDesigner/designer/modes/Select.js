dojo.provide("sketchSpaceDesigner.designer.modes.Select");

dojo.require("sketchSpaceDesigner.designer.modes.Mode");

dojo.declare("sketchSpaceDesigner.designer.modes.Select", [sketchSpaceDesigner.designer.modes.Mode], {
  enableShape: function (shape) {
    var mode = this;
    shape.onClickHandle = shape.connect("onclick", shape, function (event) { mode.onClick(shape, event); });
  },
  disableShape: function (shape) {
    dojo.disconnect(shape.onClickHandle);
  },
  onClick: function (shape, event) {
    this.designer.selection.editorShapeToggleSelection(shape, !event.ctrlKey);
  }
});
