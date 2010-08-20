import('etherpad.log');
import("sqlbase.sqlobj");
import("sqlbase.sqlbase");
import("etherpad.pad.exporthtml");
import("etherpad.utils.*");
import("etherpad.pad.padutils");
// class
function HierarchyParser(){
	
	// this basically goes down and then checks back up the chain for the closest grouping
	this.parse = function(titles, topLevelLabel,block){
		var chain = [];
		var result = { id:topLevelLabel, shortName:topLevelLabel, children:[] };
		result.path = getPath(result);
		if(block) result = block(result);
		for(var i = 1; i < titles.length; i++){
			var obj = {children:[]};
			obj.id =titles[i];
			
			for(var j = chain.length-1; j >= 0; j --){
				var link = chain[j];
				if( obj.id.match("^" + link.id)){
					link.children.push(obj);
					obj.parent = link;
					break;
				}
			}
			if(!obj.parent){
				obj.parent = result;
				result.children.push(obj);
			}
			obj.shortName = obj.id.replace(new RegExp("^" + obj.parent.id + "-"), '');
			obj.path = getPath(obj);
			if(block){
				obj = block(obj);
			}
			
			chain.push(obj);
		}
		return result;	
	}
	function getPath(obj){
		var parent = obj;
		var result = [];
		while(parent){
			result.unshift(parent.shortName);
			parent = parent.parent;
		}
		return ('/pads/' + result.join('/')).replace(/\/\//, '/'); // hacky 
	}
}

// helper methods
function getHierarchy(titles, topLevelLabel, block){
	return new HierarchyParser().parse(titles,topLevelLabel,block);
}
function getPadsBelow(top_id){
	var matching_pads = sqlobj.selectMulti("PAD_SQLMETA",{id:['like', top_id+'%']});
	return getHierarchy(matching_pads.map(function(item){ return item.id; }),top_id, function(pad){
		var json = sqlbase.getJSON("PAD_META", pad.id);
		if (json) {
			pad.meta = json;
			var html = padutils.accessPadLocal(pad.id, function(pad){
				return pad.exists() ? exporthtml.getPadHTML(pad) : null;
			}, 'r');
			pad.html = html;
		}
		return pad;
	});
}
function getImageTag(group){
	return (group.meta.images ? "<img src='" + group.meta.images[0] + "' width='60px'/>" : '');
}
function getGroupLink(group, options){
	return '<a href="'+ group.path +''+ (options && options.edit? '/+edit' : '') +'" >' + getImageTag(group) + (group.shortName || group.id || 'pads') + '</a>';
}

function getGroupChildren( group , options){
	if(group.children.length == 0){
		return getGroupLink(group, options);
	}
	var result = getGroupLink(group) + '<ul>';
	for(var i=0; i<group.children.length; i++){
		var child = group.children[i];
		result += '<li>' +getGroupChildren(child,options)  + "</li>";
	}
	return result+ '</ul>';
}
		
		