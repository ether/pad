import("etherpad.log");
import("sqlbase.sqlbase");
function padModelWriteToDB(args){
	log.info("************ in navigateByImageContentPlugin doing a save");
	var text = args.pad.text();
	var meta = args.pad._meta;
		
	//meta.images = [];// 'http://farm4.static.flickr.com/3067/2612399892_7df428d482.jpg';
	
	var REGEXP_IMG = /http.+\.(png|jpg)/igm;
	var matches = text.match(REGEXP_IMG);//REGEXP_IMG.exec(text);
	
	meta.images = matches;
	
	sqlbase.putJSON("PAD_META", meta.padId, meta);
	
	//args.pad.writeToDB(); recursive loop!!
}


function renderNavigation(args) {
	// http://localhost:9000/pads/maverick/+edit

        var parts = args.request.path.split("/pads/");
	if (parts.length < 2) return '';

	var pad_path = parts[1].split("/+edit")[0];
	var parent_path = pad_path.split("/");
	parent_path.pop();
	var pad_id = parent_path.join("-");
	
	var pads = getPadsBelow(pad_id);
	return renderTemplateAsString('imageNavigation.ejs',{pads:pads, grouped_pad_list:getGroupChildren(pads,{edit:true})}, 'navigateByImageContent');
}