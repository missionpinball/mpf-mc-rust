from server_pb2_grpc import MediaControllerStub
import grpc
from server_pb2 import SlideAddRequest, WidgetAddRequest, ShowSlideRequest

channel = grpc.insecure_channel('localhost:50051')
stub = MediaControllerStub(channel)
slide_add_request = SlideAddRequest()
new_slide = stub.AddSlide(slide_add_request)

widget_add_request = WidgetAddRequest()
widget_add_request.slide_id = new_slide.slide_id
widget_add_request.x = 5
widget_add_request.y = 5
widget_add_request.z = 2
widget_add_request.rectangle_widget.color.red = 0.0
widget_add_request.rectangle_widget.color.blue = 1.0
widget_add_request.rectangle_widget.color.green = 0.5
widget_add_request.rectangle_widget.color.alpha = 1.0
widget_add_request.rectangle_widget.width = 500
widget_add_request.rectangle_widget.height = 300
stub.AddWidgetsToSlide(widget_add_request)

widget_add_request = WidgetAddRequest()
widget_add_request.slide_id = new_slide.slide_id
widget_add_request.x = 10
widget_add_request.y = 10
widget_add_request.z = 5
#widget_add_request.video_widget.path = "/home/jan/Downloads/Biking_Girl_Alpha.mov"
widget_add_request.video_widget.path = "/home/jan/src/mpf-mc/mpfmc/tests/machine_files/video/videos/mpf_video_small_test.mp4"
stub.AddWidgetsToSlide(widget_add_request)

widget_add_request = WidgetAddRequest()
widget_add_request.slide_id = new_slide.slide_id
widget_add_request.x = 50
widget_add_request.y = 50
widget_add_request.z = 4
widget_add_request.image_widget.path = "/home/jan/src/missionpinball-website/images/mpf-flag-tiny.png"
stub.AddWidgetsToSlide(widget_add_request)

widget_add_request = WidgetAddRequest()
widget_add_request.slide_id = new_slide.slide_id
widget_add_request.x = 20
widget_add_request.y = 150
widget_add_request.z = 5
widget_add_request.label_widget.color.red = 1.0
widget_add_request.label_widget.color.blue = 0.0
widget_add_request.label_widget.color.green = 0.0
widget_add_request.label_widget.color.alpha = 1.0
widget_add_request.label_widget.text = "Hello World"
widget_add_request.label_widget.font_name = "DejaVuSerif.ttf"
widget_add_request.label_widget.font_size = 32
stub.AddWidgetsToSlide(widget_add_request)

widget_add_request = WidgetAddRequest()
widget_add_request.slide_id = new_slide.slide_id
widget_add_request.x = 30.0
widget_add_request.y = 30.0
widget_add_request.z = 6
widget_add_request.line_widget.color.red = 1.0
widget_add_request.line_widget.color.blue = 0.0
widget_add_request.line_widget.color.green = 0.0
widget_add_request.line_widget.color.alpha = 1.0
widget_add_request.line_widget.x1 = 3.0
widget_add_request.line_widget.y1 = 50.0
widget_add_request.line_widget.x2 = 150.0
widget_add_request.line_widget.y2 = 300.0
widget_add_request.line_widget.width = 10.0
stub.AddWidgetsToSlide(widget_add_request)

show_slide_request = ShowSlideRequest()
show_slide_request.slide_id = new_slide.slide_id
stub.ShowSlide(show_slide_request)
