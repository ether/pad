dojo.provide("sketchSpaceDesigner.designer.ui");

dojo.require("sketchSpaceDesigner.designer.editor");
dojo.require("sketchSpaceDesigner.designer.widgets");
dojo.require("dojo.parser");
dojo.require("dojox.layout.TableContainer");
dojo.require("dijit.layout.ContentPane");

dojo.declare("sketchSpaceDesigner.designer.DesignerUI", [dijit._Widget, dijit._Templated], {
  widgetsInTemplate: true,
  templateString: '<div>' +
		  ' <div class="topbar">' +
		  '   <div class="topbarleft"><!-- --></div>' +
		  '   <div class="topbarright"><!-- --></div>' +
		  '   <div class="topbarcenter">' +
		  '     <a href="http://github.com/redhog/pad" class="topbarBrand">SketchSpace</a>' +
		  '     <div class="fullscreen" onclick="$(\'body\').toggleClass(\'sketchSpaceMaximized\');">Full screen</div>' +
		  '     <a href="javascript:void(0);" onclick="$(\'body\').toggleClass(\'sketchSpaceMaximized\');" class="topbarmaximize" title="Toggle maximization"></a>' +
		  '   </div>' +
		  '   <div class="specialkeyarea"><!-- --></div>' +
		  ' </div>' +
		  ' <div id="sketchSpaceEditBar" dojoAttachPoint="toolbar">' +
		  '   <div class="editbar enabledtoolbar" id="editbar">' +
		  '     <div class="editbarinner" id="editbarinner">' +
		  '       <div class="editbarleft" id="editbarleft"><!-- --></div>' +
		  '       <div class="editbarright" id="editbarright"><!-- --></div>      ' +
		  '       <div class="editbarinner" id="editbarinner">' +
		  '	 <table border="0" cellspacing="0" cellpadding="0" class="editbartable" id="editbartable">' +
		  '	   <tbody><tr class="tools">' +
		  '	     <td><img height="24" width="2" src="/static/img/jun09/pad/editbar_groupleft.gif"></td>' +
		  '	     <td class="editbarbutton editbargroupsfirst tool addEllipse" unselectable="on" dojoAttachEvent="onclick:_onAddEllipse"><img title="Add ellipse" src="/static/html/plugins/sketchSpace/imgeditbar_add_circle_icon.png"></td>' +
		  '	     <td class="editbarbutton tool addPath" unselectable="on" dojoAttachEvent="onclick:_onAddPath"><img title="Add path" src="/static/html/plugins/sketchSpace/imgeditbar_add_line_icon.png"></td>' +
		  '	     <td class="editbarbutton tool addRect" unselectable="on" dojoAttachEvent="onclick:_onAddRect"><img title="Add rectangle" src="/static/html/plugins/sketchSpace/imgeditbar_add_rect_icon.png"></td>' +
		  '	     <td class="editbarbutton" unselectable="on"><img dojoAttachPoint="addImgButton" title="Add image" src="/static/html/plugins/sketchSpace/imgeditbar_add_img_icon.png"></td>' +
		  '	     <td class="editbarbutton tool select" unselectable="on" dojoAttachEvent="onclick:_onSelect"><img title="Select objects" src="/static/html/plugins/sketchSpace/imgeditbar_select_icon.png"></td>' +
		  '	     <td><img height="24" width="2" src="/static/img/jun09/pad/editbar_groupright.gif"></td>' +
		  '' +
		  '	     <td width="100%">&nbsp;</td>' +
		  '	   </tr></tbody>' +
		  '	 </table>' +
		  '	 <table border="0" cellspacing="0" cellpadding="0" class="editbarsavetable" id="editbarsavetable">' +
		  '	   <tbody><tr>' +
		  '	   </tr></tbody>' +
		  '	 </table>' +
		  '       </div>' +
		  '     </div>' +
		  '   </div>' +
		  ' </div>' +
                  '  <div id="sketchSpaceEditor" dojoAttachPoint="editorArea"></div>' +
                  '  <div id="sketchSpaceOptions" dojoType="sketchSpaceDesigner.designer.widgets.TableContainer" dojoAttachPoint="options" cols="1" showLabels="true">' +
                  '    <div dojoType="dijit.layout.ContentPane" title="Option">Value</div>' +
                  '  </div>' +
                  '</div>',
  startup: function () {
    this.inherited(arguments);

    if (typeof(AjaxUpload) != "undefined") {
      var info = {  
	action: '/ep/fileUpload/',
	name: 'uploadfile',  
	onSubmit: function(file, ext){
	//console.log('Starting...');
	},  
	onComplete: function(file, response){
	  var path = eval(response)[0].split("/");
	  sketchSpace.editorUi.addImg(path[path.length-1]);
	}
      };
      new AjaxUpload($(this.addImgButton), info);  
    }

    this.editor = new sketchSpaceDesigner.designer.editor.Editor(this.editorArea, this.attr("userId"), this, typeof(pad) == "undefined");
    var editor = this.editor;
    window.setTimeout(function () { editor.resize(); }, 1000); 

    dojo.connect(this.editor, "selectImage", this, this.onSelectImage);
    dojo.connect(this.editor, "deselectImage", this, this.onDeselectImage);

    this.selectToolIcon("select");

    if (typeof(pad) == "undefined")
      $(this.toolbar).find(".tools").css({display:"none"});
  },

  onSelectImage: function (imageId) {
    $("body").addClass("sketchSpace");
  },

  onDeselectImage: function (imageId) {
    $("body").removeClass("sketchSpace");
  },

  selectToolIcon: function(name) {
    $(this.toolbar).find(".tool").css({background: "#ffffff"});
    $(this.toolbar).find(".tool." + name).css({background: "#cccccc"});
  },

  _onAddEllipse: function() {
    this.editor.setMode(new sketchSpaceDesigner.designer.modes.AddEllipse());
    this.selectToolIcon("addEllipse");
  },

  _onAddPath: function() {
    this.editor.setMode(new sketchSpaceDesigner.designer.modes.AddPath());
    this.selectToolIcon("addPath");
  },

  _onAddRect: function() {
    this.editor.setMode(new sketchSpaceDesigner.designer.modes.AddRect());
    this.selectToolIcon("addRect");
  },

  addImg: function(imageName) {
    var shape = this.editor.createImage(this.surface_transform, imageName);
    this.editor.registerObjectShape(shape);
    this.editor.saveShapeToStr(shape);
  },

  _onSelect: function() {
    this.editor.setMode(new sketchSpaceDesigner.designer.modes.Select());
    this.selectToolIcon("select");
  },
});
