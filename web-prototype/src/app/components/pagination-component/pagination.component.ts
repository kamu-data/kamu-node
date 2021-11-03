import {
  Component,
  EventEmitter,
  Input,
  Output
} from "@angular/core";
import {PageInfoInterface} from "../../interface/search.interface";

@Component({
  selector: 'app-pagination',
  templateUrl: './pagination.component.html',
  styleUrls: ['./pagination-component.sass']
})
export class PaginationComponent {
  @Input()  public currentPage = 1;
  @Input()  public pageInfo: PageInfoInterface;
  @Output() public pageChangeEvent: EventEmitter<number> = new EventEmitter();

  public onPageChange(currentPage: number) {
    this.pageChangeEvent.emit(currentPage);
  }
}