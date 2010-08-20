import("sqlbase.sqlobj");
import("sqlbase.sqlbase");
import("etherpad.log");
import("etherpad.utils.*");
import("plugins.padHierarchy.helpers.hierarchyHelper.*");
function init() {
  this.hooks = ['renderNavigation',];
  this.aceGetFilterStack = renderNavigation;
}

function renderNavigation(){
	// http://localhost:9000/pads/maverick/+edit
	
	var pad_path = request.path.split("/pads/")[1].split("/+edit")[0];
	var parent_path = pad_path.split("/");
	parent_path.pop();
	var pad_id = parent_path.join("-");
	
	var pads = getPadsBelow(pad_id);
	return renderTemplateAsString('imageNavigation.ejs',{pads:pads, grouped_pad_list:getGroupChildren(pads,{edit:true})}, 'navigateByImageContent');
}
navigateByImageContent = new init();
