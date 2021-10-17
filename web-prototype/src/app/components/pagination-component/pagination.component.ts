import {
  AfterContentInit,
  Component,
  EventEmitter,
  Input,
  OnInit,
  Output
} from "@angular/core";
import {PageInfoInterface} from "../../interface/search.interface";

@Component({
  selector: 'app-pagination',
  templateUrl: './pagination.component.html',
  styleUrls: ['./pagination-component.sass']
})
export class PaginationComponent implements OnInit {
  @Input()  public currentPage: number = 1;
  @Input()  public pageInfo: PageInfoInterface;
  @Output() public pageChangeEvent: EventEmitter<number> = new EventEmitter();

  constructor() { }
  public ngOnInit(): void {
    debugger
  }
  public onPageChange(currentPage: number) {
    debugger
    this.pageChangeEvent.emit(currentPage);
  }
}