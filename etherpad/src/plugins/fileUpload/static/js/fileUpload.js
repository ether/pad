$(function(){  
    var btnUpload=$('#uploadFileSubmit');  

    new AjaxUpload(btnUpload, {  
        action: '/ep/fileUpload/',
        name: 'uploadfile',  
        onSubmit: function(file, ext){
	  console.log('Starting...');
        },  
        onComplete: function(file, response){  
	  console.log([response, file]);
        }  
    });  
});