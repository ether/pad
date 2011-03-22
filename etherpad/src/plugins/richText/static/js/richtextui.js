dojo.require("dijit.form.Button");
dojo.require("dijit.Menu");
dojo.require("dijit.ColorPalette");
dojo.require('dijit.form.Select');



function colorPalette(name, img, callback){ 
	//html && dom string must follow a sepecify rule see the html content in the test file
	var Palette = new dijit.ColorPalette({
                palette: "7x10",
                onChange: function(val) {
        		dojo.style(name + "ColorIndicator", "backgroundColor", val);
       	        	Button.set("value", val);
	        	if(callback){
			   callback(val);
			}
                }
              },
              name + "CPplaceHolder");

        var Button = new dijit.form.ComboButton({
               label: "<span style='position:relative'><img src=' " + img + "' />\
          			<span id='"+ name +"ColorIndicator'style='background-color:#000;display:block;\
		        	 position:absolute;height:5px; top:11px; width:100%'>&nbsp;</span></span>",
               name: name + "cp",
    	       value : "#000000",
               dropDown: Palette,
		       onClick : function(){
			       if(callback){
        			   callback(this.get("value"));
		           }
		       },
               id: name + "cpButton"
        });
        dojo.byId(name + "ColorPalette").appendChild(Button.domNode);
}
function buildColorPalette(){ //build text-color and background-color palette
	colorPalette("text", "/static/img/plugins/richText/textcolor.gif", function(val){
          richTextexecCommand("color", val);
     });
	colorPalette("bg", "/static/img/plugins/richText/bgcolor.gif", function(val){
          richTextexecCommand("backgroundColor", val);
     });
}

function styleMenuList(name, style, values, callback, selectedIndex){
    var menu = new dijit.Menu({
         //  style: "display: none; width:100px",
         style: { width: '100px', overflow:"hidden" },
         forceWidth : true
    });
	selectedIndex = selectedIndex || 0;
	var strList = [
	   "<span style='",
	   style,
	   ":",
	   "", //value
	   "'>",
	   "", //value
	   "</span>" 		  
	];
	for(var i = 0, len = values.length; i < len; i++){
		strList[3] = strList[5] = values[i];
        var menuItem = new dijit.MenuItem({
	          label : strList.join(""), 
	 	      value : values[i],
              onClick: function() {
		      var val = this.get("value");		
		    	   button.set({"label":val, "value": val});
			       if(callback){
				        callback(val);
    			   }
	          }
       });
       menu.addChild(menuItem);
	}

    var button = new dijit.form.DropDownButton({
          label: values[selectedIndex],
	      value: values[selectedIndex], 
          name: name + "menulist",
          dropDown: menu,
          id: name + "menu"
    });
    dojo.byId(name + "placeholder").appendChild(button.domNode)
} 
	
function familyLists(values, callback, selectedIndex){
	var strList = [
	   "<span style='",
	   "font-family",
	   ":",
	   "", //value
	   "'>",
	   "", //value
	   "</span>" 		  
	];
	var opts = [];
	selectedIndex = selectedIndex || 0;
	for(var i = 0, len = values.length; i < len; i++){
		strList[3] = strList[5] = values[i];
		opts.push({
			label : strList.join(""),
			value : values[i]	
		});
	}	
	strList[3] = strList[5] = values[selectedIndex];
	var select = new dijit.form.Select({
	    id: 'fontfamilysetter',
		label : strList.join(""),
		value : values[selectedIndex],
	    style: { width: '130px', overflow:"hidden" },
	    forceWidth : true,
		onChange : function(val){
			if(callback){
			  callback(val);
			}
		},
	    options: opts
    }).placeAt("fontfamilyplaceholder");
}

function buildFontStyle(){
	var sizes = ["10px", "12px", "16px", 
			"18px", "24px", "32px", "48px"];
	var families = ["Arial", "Arial Black", "Comic Sans MS", 
			"Georgia", "Times New Roman", "\u5b8b\u4f53", "\u6977\u4f53"]; 
    styleMenuList("fontsize", "font-size", sizes, function(val){
          richTextexecCommand("fontSize", val);
    });	
	//styleMenuList("fontfamily", "font-family",
	//families, function(val){alert(val)}); //how to set fix width in dojo?
	familyLists(families, function(val){
          richTextexecCommand("fontFamily", val);
    });
}
	
dojo.addOnLoad(function() {
	buildColorPalette();
	buildFontStyle();
});
