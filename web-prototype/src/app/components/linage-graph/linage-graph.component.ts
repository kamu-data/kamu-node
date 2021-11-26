import {Component, EventEmitter, Input, Output} from "@angular/core";

@Component({
  selector: 'app-linage-graph',
  templateUrl: './linage-graph.component.html',
})
export class LinageGraphComponent {
  @Input() public view: [number, number];
  @Input() public links: any[];
  @Input() public nodes: any[];

  @Output() public onClickNodeEvent: EventEmitter<string> = new EventEmitter();

  public draggingEnabled = false;
  public panningEnabled = true;
  public zoomEnabled = true;
  public zoomSpeed = 0.1;
  public minZoomLevel = 0.1;
  public maxZoomLevel = 4.0;
  public panOnZoom = true;
  public autoZoom = true;
  public autoCenter = false;
  public showMiniMap = true;
  public miniMapPosition: any;

  public onClickNode(label: string): void {
    this.onClickNodeEvent.emit(label);
  }
}
