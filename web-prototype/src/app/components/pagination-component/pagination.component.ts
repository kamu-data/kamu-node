import {
  Component,
  EventEmitter,
  Input,
  OnInit,
  Output
} from "@angular/core";

@Component({
  selector: 'app-pagination',
  templateUrl: './pagination.component.html',
  styleUrls: ['./pagination-component.sass']
})
export class PaginationComponent implements OnInit {
  public page: number = 1;
  constructor() { }
  public ngOnInit(): void {
  }
  public onPageChange() {
    debugger
  }
}