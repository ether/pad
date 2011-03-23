dojo.provide("sketchSpaceDesigner.designer.selection");

dojo.declare("sketchSpaceDesigner.designer.selection.Selection", [], {
  constructor: function (designer) {
    this.designer = designer;
    this.objects = {};
    this.parent = undefined;
    this.outline = undefined;
  }
});
