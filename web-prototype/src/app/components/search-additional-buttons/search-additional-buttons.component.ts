import {Component, EventEmitter, Input, OnInit, Output} from '@angular/core';
import {SearchAdditionalButtonInterface} from "./search-additional-buttons.interface";

@Component({
  selector: 'search-additional-buttons',
  templateUrl: './search-additional-buttons.component.html',
  styleUrls: ['./search-additional-buttons.component.sass']
})
export class SearchAdditionalButtonsComponent implements OnInit {
  @Input() public searchAdditionalButtonsData: SearchAdditionalButtonInterface[];
  @Input() public isMinimizeSearchAdditionalButtons: boolean;
  @Output() public searchAdditionalButtonsMethod: EventEmitter<string> = new EventEmitter();
  constructor() { }

  public ngOnInit(): void {
  }
  public onClick(method: string): void {
    this.searchAdditionalButtonsMethod.emit(method);
  }

}
