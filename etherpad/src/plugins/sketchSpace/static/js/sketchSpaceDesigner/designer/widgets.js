dojo.provide("sketchSpaceDesigner.designer.widgets");

dojo.require("dojox.widget.ColorPicker");

dojo.declare("sketchSpaceDesigner.designer.widgets.ColorPickerPopup", [dojox.widget.ColorPicker], {
  create: function () {
    this.inherited(arguments);
    dijit.popup.moveOffScreen(this.domNode);
  },
  popup: function () {
    var widget = this;
    dijit.popup.open({
      parent: null,
      popup: widget,
      around: widget.popupFor,
      orient: {'BR':'TR', 'BL':'TL', 'TR':'BR', 'TL':'BL'},
      onExecute: function(){
	dijit.popup.close(widget);
        widget.setColor(widget.attr("value"));
      },
      onCancel: function(){ dijit.popup.close(widget); },
      onClose: function(){}
    });
    this.focus();
  },
  setColor: function(colorHex) {
    this.inherited(arguments);
    dojo.style(this.popupFor, "background", colorHex);
  },
  onBlur: function () {
    this.inherited(arguments);
    this.onCancel();
  },
  onCancel: function () {},
});
