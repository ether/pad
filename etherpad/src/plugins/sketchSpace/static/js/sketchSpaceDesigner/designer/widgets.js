dojo.provide("sketchSpaceDesigner.designer.widgets");

dojo.require("dojox.widget.ColorPicker");
dojo.require("dojox.layout.TableContainer");
dojo.require("dijit.form.CheckBox");
dojo.require("dijit.form.NumberSpinner");
dojo.require("dijit.form.NumberTextBox");

dojo.declare("sketchSpaceDesigner.designer.widgets.ColorPickerPopup", [dojox.widget.ColorPicker], {
  create: function () {
    this.inherited(arguments);
    dijit.popup.moveOffScreen(this.domNode);
  },
  popup: function (popupFor, setColor) {
    var widget = this;
    dijit.popup.open({
      parent: null,
      popup: widget,
      around: popupFor,
      orient: {'BR':'TR', 'BL':'TL', 'TR':'BR', 'TL':'BL'},
      onExecute: function(){
       // dijit.popup.close(widget);
        setColor(widget.attr("value"));
      },
      onCancel: function(){ dijit.popup.close(widget); },
      onClose: function(){}
    });
    this.focus();
  },
  onBlur: function () {
    this.inherited(arguments);
    this.onCancel();
  },
  onCancel: function () {},
});

dojo.declare("sketchSpaceDesigner.designer.widgets.ColorInput", [dijit.form._FormValueWidget, dijit._Templated], {
  widgetsInTemplate: true,
  value: "#ff0000",
  templateString: '<span style="display: inline-block; width: 10pt; height: 10pt; border: 2px solid black; " dojoAttachEvent="onclick:_onClick" dojoAttachPoint="focusNode">' + 
                  '  <input dojoAttachPoint="valueNode" type="hidden" ${!nameAttrSetting} />' +
                  '  <span dojoType="sketchSpaceDesigner.designer.widgets.ColorPickerPopup" dojoAttachPoint="popup"></span>' +
                  '</span>',
  _setValueAttr: function(value, priorityChange){
    if (value === undefined)
      value = "";
    else if (value.toHex !== undefined)
      value = value.toHex();
    else if (value.r !== undefined)
      value = new dojox.color.Color(value).toHex();

    last_value = value;
    dojo.style(this.domNode, "background", value);
    this.inherited(arguments, [value]);
  },
  _getValueAttr: function(){
    return dojox.color.fromHex(this.inherited(arguments));
  },
  _onClick: function (event) {
    var widget = this;
    this.popup.setColor(this.attr("value").toHex());
    this.popup.popup(this.domNode, function (color) { widget.attr("value", color); });
  },
});


dojo.declare("sketchSpaceDesigner.designer.widgets.OptionInput", [], {
  startup: function () {
    this.inherited(arguments);
    this.setAttrFromOptions(); // force an update from options
    this.setOptionsHandle = dojo.connect(this.attr("designer"), "setOptions", this, this.setAttrFromOptions);
    this.isStarted = true;
  },
  destroy: function () {
    dojo.disconnect(this.setOptionsHandle); 
    this.inherited(arguments);
  },
  setAttrFromOptions: function () {
    this.inhibitSetOptions = true;
    this.attr("value", this.attr("designer").getOptionByPath(this.attr("optionsPath")));
    this.inhibitSetOptions = false;
  },
  setOptionsFromAttr: function (value) {
    if (!this.inhibitSetOptions && this.isStarted) {
      var options = {};
      options[this.attr("optionsPath")] = value;
      this.attr("designer").setOptionsByPath(options);
    }
    return this.inherited(arguments);
  },
  _setValueAttr: function(value, priorityChange){
    this.setOptionsFromAttr(value);
    return this.inherited(arguments);
  },
  _setCheckedAttr: function(value, priorityChange){
    this.setOptionsFromAttr(value);
    return this.inherited(arguments);
  },
});

dojo.declare("sketchSpaceDesigner.designer.widgets.ColorOptionInput", [sketchSpaceDesigner.designer.widgets.ColorInput, sketchSpaceDesigner.designer.widgets.OptionInput], {});
dojo.declare("sketchSpaceDesigner.designer.widgets.OptionCheckBox", [dijit.form.CheckBox, sketchSpaceDesigner.designer.widgets.OptionInput], {});
dojo.declare("sketchSpaceDesigner.designer.widgets.OptionNumberSpinner", [dijit.form.NumberSpinner, sketchSpaceDesigner.designer.widgets.OptionInput], {});
dojo.declare("sketchSpaceDesigner.designer.widgets.OptionNumberTextBox", [dijit.form.NumberTextBox, sketchSpaceDesigner.designer.widgets.OptionInput], {});

/* Bug workaround */
dojo.declare("sketchSpaceDesigner.designer.widgets.TableContainer", [dojox.layout.TableContainer], {
  layout: function () {
    this._children = [];
    return this.inherited(arguments);
  }
});
