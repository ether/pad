dojo.provide("sketchSpaceDesigner.designer.modes.Select");

dojo.require("sketchSpaceDesigner.designer.modes.Zoom");

dojo.declare("sketchSpaceDesigner.designer.modes.Select", [sketchSpaceDesigner.designer.modes.Zoom], {
  enable: function () {
    var mode = this;
    this.inherited(arguments);
    this.onKeyUpHandle = dojo.connect(document, "onkeyup", this, function (event) { mode.onKeyUp(event); });
  },
  disable: function () {
    this.inherited(arguments);
    dojo.disconnect(this.onKeyUpHandle);
  },
  enableShape: function (shape) {
    var mode = this;
    shape.onClickHandle = shape.connect("onclick", shape, function (event) { mode.onClick(shape, event); });
  },
  disableShape: function (shape) {
    dojo.disconnect(shape.onClickHandle);
  },
  onClick: function (shape, event) {
    this.designer.selection.editorShapeToggleSelection(shape, !event.ctrlKey);
  },
  onKeyUp: function (event) {
    if (event.keyCode == 46)
      this.designer.selection.editorSelectionShapeRemove();
  }
});
