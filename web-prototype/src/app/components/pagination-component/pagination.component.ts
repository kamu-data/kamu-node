import {
  Component,
  EventEmitter,
  Input, OnChanges,
  Output, SimpleChanges
} from "@angular/core";
import {PageInfoInterface} from "../../interface/search.interface";

@Component({
  selector: 'app-pagination',
  templateUrl: './pagination.component.html',
  styleUrls: ['./pagination-component.sass']
})
export class PaginationComponent implements OnChanges {
  @Input()  public currentPage: number;
  @Input()  public pageInfo: PageInfoInterface;
  @Output() public pageChangeEvent: EventEmitter<{currentPage: number, isClick: boolean}> = new EventEmitter();

  private previousPage: number;

  public ngOnChanges(changes: SimpleChanges): void {
    const page = changes.currentPage;
    if (!page && !this.currentPage) {
      this.previousPage = 1;
      this.currentPage = 1;
    }

    if (page && !page.previousValue && page.firstChange) {
      this.previousPage = page.currentValue;
    }
    if (page && page.currentValue) {
      this.previousPage = page.currentValue;
      this.currentPage = page.currentValue;
    }
  }

  public onPageChange(currentPage: number, isClick: boolean = false) {
    if (currentPage !== this.previousPage) {
      this.previousPage = currentPage;
      this.pageChangeEvent.emit({currentPage, isClick});
    }
  }
}