import("etherpad.utils.*");
import("etherpad.log");
import("etherpad.control.pad.pad_control");
import("sqlbase.sqlobj");
import("sqlbase.sqlbase");
import("etherpad.pad.exporthtml");
import("etherpad.pad.model");
import("etherpad.pad.padutils");
function onRequest() {
	var section = request.path.toString().split("/specs/")[1];
	var filter = section.replace(/\//g,"-");
	function shortName(pad_id){
		return pad_id.replace(filter, '');
	}
	function groupBasedId(pad_id){
		return request.path + shortName(pad_id);
	}
	
	var matching_pads = sqlobj.selectMulti("PAD_SQLMETA",{id:['like', filter+'%']});
	var pads = [];
	// seems like too many db queries - is there a selectMultiJSON command thing? getAllJSON? Does that support filter conditions?
	for (var i in matching_pads) {
		var json= sqlbase.getJSON("PAD_META", matching_pads[i].id);
		if(json) pads.push(json);
	}
	
	var summary_pad_id = request.path.replace(/^\/specs\//,'').replace(/\/$/,'').replace(/\//g, '-');
	
	var summary = 
	padutils.accessPadLocal(summary_pad_id, function(pad){
		return pad.exists() ? exporthtml.getPadHTML( pad ) : null;	
	}, 'r');
	 
	
	renderHtml('groupIndex.ejs',{	summary_pad_id:summary_pad_id,
									summary:summary,
									section:section,
									filter:filter, 
									pads:pads,
									groupBasedId:groupBasedId,
									shortName:shortName},'supportGroupUrls');
	return true;
}
function render_page(){
	var padId = request.path.toString().split("/specs/")[1].replace(/\//g,"-");
	log.info("****rendering  " + padId);
	return pad_control.render_pad(padId);
}
function redirect_to_specs_path(){
	response.redirect("/specs" + request.path);
	
}
